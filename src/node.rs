use std::collections::HashMap;
use std::sync::Arc;
use crate::database::{bstate, BStatus};
use anyhow::Result;
use axum::http::status;
use log::info;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tokio::select;
use reqwest;
use crate::routes::currstatus::CurrentStatusResponse;


pub struct Node {
    // pub bstatus: Arc<RwLock<BStatus>>,
    // pub port: u16,
    // pub known_peers: Arc<RwLock<HashMap<String,PeerNode>>>,
    pub bstatus:BStatus,
    pub port: u16,
    pub known_peers:HashMap<String,PeerNode>,
}

impl Node{
    pub fn new(port: u16, bstatus: BStatus, bootstrap : PeerNode) -> Result<Self> {
        let mut peers=HashMap::new();
        let tcp_addr=bootstrap.tcp_addr();
        peers.insert(tcp_addr,bootstrap);

        Ok(
            Self {
                bstatus: bstatus,
                port,
                known_peers: peers,
            }
        )
    }
    pub async  fn sync(&mut self) -> Result<()> {
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

    pub async fn fetch_new_block_and_peer(&mut self) -> Result<()> {
        info!("fetching new block and peer");
        let mut new_peers=HashMap::new();
        let bstatus = &self.bstatus;
        for (tcp_addr, peer) in self.known_peers.iter(){

            let peer_curr_state =query_peer_status(peer).await?; // 错误也要继续
            let  lock_block_number =bstatus.get_last_block().header.number;
            if peer_curr_state.height > lock_block_number {
                let block_count = peer_curr_state.height - lock_block_number;
                info!("need fetching {:?}new block from peer{:?}:", block_count, tcp_addr);
                //self.bstatus.fetch_new_block(&bstate.hash).await?;
            }

            for (tcp_addr, peer) in peer_curr_state.known_peers.iter(){
                if !self.known_peers.contains_key(tcp_addr){
                    info!("found new peer: {:?}", tcp_addr);
                    new_peers.insert(tcp_addr.clone(),peer.clone());
                }
            }
        }
        for (tcp_addr, peer) in new_peers.iter(){
            self.known_peers.insert(tcp_addr.clone(),peer.clone());
        }
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
    info!("querying peer status: {:?}", peer);
    let tcp_addr = peer.tcp_addr();
    let client = reqwest::Client::new();
    let url = format!("http://{}/node/status", tcp_addr);
    let res :CurrentStatusResponse = client.get(&url).send().await?.json::<CurrentStatusResponse>().await?;

    //let bs=serde_json::from_value(res)?;
    info!("peer status: {:?}", res);
    Ok(res)
}
