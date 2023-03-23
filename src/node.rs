use std::collections::HashMap;
use std::sync::Arc;
use crate::database::{ BStatus};
use anyhow::Result;
use log::info;
use tokio::sync::{RwLock};
//use parking_lot::RwLock ;
use serde::{Deserialize, Serialize};
use tokio::select;
use reqwest;
use crate::routes::currstatus::CurrentStatusResponse;

#[derive(Debug, Clone)]
pub struct Node {
    pub dir_path: String,

    pub bstatus: Arc<RwLock<BStatus>>,
    pub known_peers: Arc<RwLock<HashMap<String,PeerNode>>>,
}

impl Node{
    pub fn new(dir_path:String, bootstrap : PeerNode) -> Result<Self> {
        let bstatus = BStatus::new(dir_path.clone())?;
        let mut peers=HashMap::new();
        let tcp_addr=bootstrap.tcp_addr();

        info!("node created, bootstrap node ip  {}",tcp_addr);
        peers.insert(tcp_addr,bootstrap);

         let node=   Self {
                dir_path,
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
    pub async fn set_known_peers(&self, peers: HashMap<String,PeerNode>) {
        let mut known_peers = self.known_peers.write().await;
        for (tcp_addr, peer) in peers.iter(){
            known_peers.insert(tcp_addr.clone(),peer.clone());
        }
    }

    pub async fn sync(&self) -> Result<()> {
        info!("syncing with peers");
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(45));
        loop {
            select! {
                _ = ticker.tick() => {
                    self.fetch_new_block_and_peer().await?;
                }
            }

        }
    }

    pub async fn fetch_new_block_and_peer(&self) -> Result<()> {
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
        self.set_known_peers(new_peers).await;
        Ok(())
    }

}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PeerNode {
    pub ip: String,
    pub port: u16,
    pub is_bootstrap: bool,
    pub is_active: bool,
}

impl PeerNode {
    pub fn new(ip: String, port: u16, is_bootstrap: bool, is_active: bool) -> Self {
        Self {
            ip,
            port,
            is_bootstrap,
            is_active,
        }
    }
    fn tcp_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

async fn query_peer_status(peer: &PeerNode) -> Result<CurrentStatusResponse> {
    info!("querying peer status from node: {:?}", peer.tcp_addr());
    let tcp_addr = peer.tcp_addr();
    let url = format!("http://{}/node/status", tcp_addr);
    let client = reqwest::Client::new();
    let resp :CurrentStatusResponse = client.get(&url).send().await.unwrap().json::<CurrentStatusResponse>().await?;
    // let resp = reqwest::blocking::get(url)?
    //     .json::<CurrentStatusResponse>()?;
    //
    Ok(resp)
}
