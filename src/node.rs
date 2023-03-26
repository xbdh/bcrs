use std::collections::HashMap;
use std::sync::Arc;
use crate::database::{BHash, Block, BStatus, Tx};
use anyhow::{anyhow, Result};
use log::info;
use tokio::sync::{Mutex, RwLock};
//use parking_lot::RwLock ;
use serde::{Deserialize, Serialize};
use tokio::select;
use reqwest;
use crate::routes::addpeer::AddPeerResponse;
use crate::routes::currstatus::CurrentStatusResponse;
use tokio::sync::mpsc::{Receiver, Sender};
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};

const SYNC_INTERVAL: u64 = 25;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PeerNode {
    pub ip: String,
    pub port: u16,
    pub is_bootstrap: bool,

    // when peer is connected to this node, it will be set to true
    pub  is_connected :bool,
}

impl PeerNode {
    pub fn new(ip: String, port: u16, is_bootstrap: bool, is_connected: bool) -> Self {
        Self {
            ip,
            port,
            is_bootstrap,
            is_connected,
        }
    }
    pub(crate) fn tcp_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}



#[derive(Debug)]
pub struct CancelFlag {
    cancel_flag: AtomicBool,
}

impl CancelFlag {
    pub fn new() -> Self {
        Self {
            cancel_flag: AtomicBool::new(false),
        }
    }
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }
    pub fn is_canceled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }
}


// new sync block channel struct,contains mspc channel reciver and sender
#[derive(Debug)]
pub struct SyncBlockChannel {
    pub receiver: Arc<Mutex<Receiver<Block>>>,
    pub sender: Arc<Mutex<Sender<Block>>>   ,
}



impl SyncBlockChannel{
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        Self {
            receiver: Arc::new(Mutex::new(receiver)),
            sender: Arc::new(Mutex::new(sender)),
        }
    }
    pub async fn send(&self,block:Block) -> Result<()> {
        let mut sender = self.sender.lock().await;
        sender.send(block).await?;
        Ok(())
    }
    pub async fn recv(&self) -> Result<Block> {
        let mut receiver = self.receiver.lock().await;
        let block = receiver.recv().await.ok_or(anyhow!("recv block error"))?;
        Ok(block)
    }
}




#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub dir_path: String,
    pub info :PeerNode,

    pub bstatus: Arc<RwLock<BStatus>>,
    pub known_peers: Arc<RwLock<HashMap<String,PeerNode>>>,

    pub pending_txs: Arc<RwLock<HashMap<BHash,Tx>>>,
    pub archive_txs: Arc<RwLock<HashMap<BHash,Tx>>>,

    pub new_sync_block_channel: Arc<SyncBlockChannel>,
    pub cancel_flag: Arc<CancelFlag>,
    pub is_mining: Arc<RwLock<bool>>,

}

impl Node{
    pub fn new(name:String,dir_path:String,ip:String,port:u16, bootstrap : PeerNode) -> Result<Self> {
        let bstatus = BStatus::new(dir_path.clone())?;
        let mut peers=HashMap::new();
        let tcp_addr=bootstrap.tcp_addr();

        let mut is_bootstrap = false;
        let mut btc=bootstrap.clone();
        if bootstrap.ip==ip && bootstrap.port==port {
            info!("{} created, bootstrap node is self",name);
            is_bootstrap = true;
            btc.is_connected=true;
        }
        info!("bootstrap node ip  {}",tcp_addr);
        peers.insert(tcp_addr,btc);

         let node=   Self {
                name,
                dir_path,
                info: PeerNode::new(ip,port,is_bootstrap,false),
                bstatus: Arc::new(RwLock::new(bstatus)),
                known_peers: Arc::new(RwLock::new(peers)),
                pending_txs: Arc::new(RwLock::new(HashMap::new())),
                archive_txs: Arc::new(RwLock::new(HashMap::new())),
                new_sync_block_channel: Arc::new(SyncBlockChannel::new()),
                cancel_flag: Arc::new(CancelFlag::new()),
                is_mining: Arc::new(RwLock::new(false)),
         };

        Ok(node)

    }
    pub async fn get_status(&self) -> BStatus {
        let  b= self.bstatus.read().await;
        b.clone()
    }
    pub async fn get_known_peers(&self) -> HashMap<String,PeerNode> {
        let known_peers = self.known_peers.read().await;
        known_peers.clone()
    }
    pub async fn add_many_to_known_peers(&self, peers: HashMap<String,PeerNode>) {
        let mut known_peers = self.known_peers.write().await;
        for (tcp_addr, peer) in peers.iter(){
            known_peers.insert(tcp_addr.clone(),peer.clone());
        }
    }
    pub async fn add_peer_to_known_peers(&self, tcp_addr: String, peer: PeerNode) {
        let mut known_peers = self.known_peers.write().await;
        known_peers.insert(tcp_addr,peer);
    }
    pub async fn remove_peer_from_known_peers(&self, tcp_addr: String) {
        let mut known_peers = self.known_peers.write().await;
        known_peers.remove(&tcp_addr);
    }

    pub async fn get_pending_txs(&self) -> HashMap<BHash,Tx> {
        let pending_txs = self.pending_txs.read().await;
        pending_txs.clone()
    }

    pub async fn add_tx_to_pending_txs(&self, tx: Tx) {
        let mut pending_txs = self.pending_txs.write().await;
        pending_txs.insert(tx.tx_hash(),tx);
    }
    pub async fn remove_tx_from_pending_txs(&self, tx_hash: BHash) {
        let mut pending_txs = self.pending_txs.write().await;
        pending_txs.remove(&tx_hash);
    }

    pub async fn get_archive_txs(&self) -> HashMap<BHash,Tx> {
        let archive_txs = self.archive_txs.read().await;
        archive_txs.clone()
    }
    pub async fn add_tx_to_archive_txs(&self, tx: Tx) {
        let mut archive_txs = self.archive_txs.write().await;
        archive_txs.insert(tx.tx_hash(),tx);
    }
    pub async fn remove_tx_from_archive_txs(&self, tx_hash: BHash) {
        let mut archive_txs = self.archive_txs.write().await;
        archive_txs.remove(&tx_hash);
    }

    pub async fn get_is_mining(&self) -> bool {
        let is_mining = self.is_mining.read().await;
        *is_mining
    }
    pub async fn set_is_mining(&self, is_mining_ing: bool) {
        let mut is_mining = self.is_mining.write().await;
        *is_mining = is_mining_ing;
    }


    pub async fn get_last_block_hash(&self) -> BHash {
        let bstatus = self.bstatus.read().await;
        bstatus.get_last_block_hash()
    }
    pub async fn get_next_block_number(&self) -> u64 {
        let bstatus = self.bstatus.read().await;
        if bstatus.last_block_hash==BHash::default() && bstatus.get_height()==0{
            return 0;
        }
        bstatus.get_height()+1
    }

    // pub async fn get_sync_block_channel(&self) -> SyncBlockChannel {
    //     let new_sync_block_channel = self.new_sync_block_channel.lock().await;
    //     new_sync_block_channel.clone()
    // }




}


