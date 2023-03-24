use std::collections::HashMap;
use std::sync::Arc;
use crate::database::{BHash, Block, BStatus};
use anyhow::{anyhow, Result};
use log::info;
use tokio::sync::{RwLock};
//use parking_lot::RwLock ;
use serde::{Deserialize, Serialize};
use tokio::select;
use reqwest;
use crate::routes::addpeer::AddPeerResponse;
use crate::routes::currstatus::CurrentStatusResponse;

const ENDOINT_STATUS: &str = "/node/status";

const ENDOINT_SYNC: &str = "/node/sync";
const ENDOINT_SYNC_QUERY_KEY: &str = "begin_hash";

const ENDOINT_ADD_PERR: &str = "/node/peer";
const ENDOINT_ADD_PERR_QUERY_KEY_IP: &str = "ip";
const ENDOINT_ADD_PERR_QUERY_KEY_PORT: &str = "port";

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub is_bootstrap: bool,
    pub dir_path: String,
    pub ip: String,
    pub port: u16,
    pub bstatus: Arc<RwLock<BStatus>>,
    pub known_peers: Arc<RwLock<HashMap<String,PeerNode>>>,
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
                is_bootstrap,
                dir_path,
                ip,
                port,
                bstatus: Arc::new(RwLock::new(bstatus)),
                known_peers: Arc::new(RwLock::new(peers)),
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
    pub async fn add_blocks(&self, block:Vec< Block>) -> Result<()> {
        let mut bstatus = self.bstatus.write().await;
        for b in block.iter() {
           bstatus.add_block(b.clone())?;
        }
        Ok(())
    }
    async fn is_known_peer(&self, peer:&PeerNode) -> bool {
        // is myself
        if peer.ip==self.ip&&peer.port==self.port{
            return true;
        }
        let known_peers = self.get_known_peers().await;
        let tcp_addr=peer.tcp_addr();
        known_peers.contains_key(&tcp_addr)
    }

    pub async fn sync(&self) -> Result<()> {
        info!("syncing with peers");
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(45));
        loop {
            select! {
                _ = ticker.tick() => {
                    self.do_sync().await?;
                }
            }

        }
    }

    pub async fn do_sync(&self) -> Result<()> {
        info!("do synce");
        for (tcp_addr, peer) in self.get_known_peers().await.iter(){
          // 错误也要继续
            let peer_curr_state =  match query_peer_status(peer).await{
                Ok(peer_curr_state) => {
                    peer_curr_state
                }
                Err(e) => {
                    info!("query peer status error: {:?}", e);
                    self.remove_peer_from_known_peers(tcp_addr.clone()).await;
                    continue;
                }
            };
            match self.join_know_peers(peer).await{
                Ok(_) => {
                    info!("join peer success");
                }
                Err(e) => {
                    info!("join peer error: {:?}", e);
                    continue;
                }
            }
            match self.sync_blocks(peer,&peer_curr_state).await{
                Ok(_) => {
                    info!("sync blocks success");
                }
                Err(e) => {
                    info!("sync blocks error: {:?}", e);
                    continue;
                }
            }

            match self.sync_known_peers(peer,&peer_curr_state).await{
                Ok(_) => {
                    info!("sync peers success");
                }
                Err(e) => {
                    info!("sync peers error: {:?}", e);
                    continue;
                }
            }
        }
        Ok(())
    }

    pub async fn join_know_peers(&self, peer:&PeerNode) -> Result<()> {
        info!("join peer: {:?}", peer);
        if peer.is_connected{
            return Ok(());
        }
        let url = format!("http://{}:{}{}?{}={}&{}={}",
                            peer.ip,
                            peer.port,
                            ENDOINT_ADD_PERR,
                            ENDOINT_ADD_PERR_QUERY_KEY_IP,
                            self.ip,
                            ENDOINT_ADD_PERR_QUERY_KEY_PORT,
                            self.port);
        //info!("join peer url: {}",url);
        let client = reqwest::Client::new();
        let res = client.get(&url).send().await?;
        let res: AddPeerResponse = res.json().await?;
        if res.success==false{
          return Err(anyhow!("add peer error"));
        }
        // 对方把我加入到对方的peer列表中，我也要把对方加入到我的peer列表中
        let known_peers = self.get_known_peers().await;
        let tcp_addr=peer.tcp_addr();
        let mut peerc=peer.clone();
        peerc.is_connected=true;
       // if !known_peers.contains_key(&tcp_addr){
       //      self.add_peer_to_known_peers(tcp_addr, peer).await;
       //  }
        self.add_peer_to_known_peers(tcp_addr, peerc).await;
        Ok(())
    }

    pub async fn sync_blocks(&self, peer:&PeerNode,node_status:&CurrentStatusResponse) -> Result<()> {
        info!("sync blocks from peer: {:?}", peer);
        let bstatus = self.get_status().await;
        let local_block_height =bstatus.get_height();
        let local_block_hash =bstatus.get_last_block_hash();

        // 如果本地高度大于等于对方高度，不需要同步
        info!("local_block_height:{:?},node_status.height:{:?}",local_block_height,node_status.height);

        // 如果本地为没有块，对方有至少一块，需要同步，从对方的第一块开始同步，begin_hash=Default::default()
        if local_block_height==0&&local_block_hash==Default::default()&&node_status.hash!=Default::default(){
            info!("local_block_is empty ,async from first block");
            let blocks= fetch_blocks(peer,Default::default()).await?;
            self.add_blocks(blocks).await?;
            return Ok(());
        }

        if local_block_height<node_status.height{
            info!("need async {} block from other",node_status.height-local_block_height);
            let blocks= fetch_blocks(peer,local_block_hash).await?;
            self.add_blocks(blocks).await?;
            return Ok(());
        }

        Ok(())
    }
    
    pub async fn sync_known_peers(&self,peer:&PeerNode,node_status:&CurrentStatusResponse) -> Result<()> {
        for (tcp_addr,peer) in node_status.known_peers.iter(){
            if !self.is_known_peer(peer).await{
                self.add_peer_to_known_peers(peer.tcp_addr(), peer.clone()).await;
            }
        }
        Ok(())
    }
    pub async fn fetch_new_block_from_peer(&self) -> Result<()> {
        info!("fetching new block and peer");
        let mut new_peers=HashMap::new();

        let known_peers = self.get_known_peers().await;
        let bstatus = self.get_status().await;


        for (tcp_addr, peer) in known_peers.iter(){

            let peer_curr_state =query_peer_status(peer).await?; // 错误也要继续

            let  lock_block_number =bstatus.get_last_block().header.number;
            if peer_curr_state.height > lock_block_number {
                let block_count = peer_curr_state.height - lock_block_number;
                info!("need fetching {:?}new block from peer{:?}:", block_count, tcp_addr);
                //self.bstatus.fetch_new_block(&bstate.hash).await?;
            }

            for (tcp_addr, peer) in peer_curr_state.known_peers.iter(){
                if !known_peers.contains_key(tcp_addr){
                    info!("found new peer: {:?}", tcp_addr);
                    new_peers.insert(tcp_addr.clone(),peer.clone());
                }
            }
        }
        self.add_many_to_known_peers(new_peers).await;
        Ok(())
    }
    

}

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

async fn query_peer_status(peer: &PeerNode) -> Result<CurrentStatusResponse> {
    info!("querying peer status from node: {:?}", peer.tcp_addr());
    let tcp_addr = peer.tcp_addr();
    let url = format!("http://{}{}", tcp_addr,ENDOINT_STATUS);
    //info!("querying peer ：url: {:?}", url);
    let client = reqwest::Client::new();
    // 处理掉线情况
    match client.get(&url).send().await {
        Ok(resp) => {
            let resp: CurrentStatusResponse = resp.json().await?;
           Ok(resp)
        }
        Err(e) => {
            info!("query peer status error: cant connect peer", );
            Err(anyhow!("query peer status error"))
        }
    }

}
pub async fn fetch_blocks(peer: &PeerNode,begin_hash:BHash) -> Result<Vec<Block>> {
    info!("fetching blocks from node: {:?}", peer.tcp_addr());
    let tcp_addr = peer.tcp_addr();
    let begin_hash_str = hex::encode(begin_hash.0);

    let url = format!("http://{}{}?{}={}",
                      tcp_addr,
                      ENDOINT_SYNC,
                      ENDOINT_SYNC_QUERY_KEY,
                      begin_hash_str);
    info!("fetching blocks ：url: {:?}", url);

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.unwrap().json::<Vec<Block>>().await?;
    Ok(resp)
}
