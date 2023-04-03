use crate::node::{Node, PeerNode};
use log::info;
use anyhow::{anyhow, Result};
use tokio::select;
use crate::database::{BHash, Block, Tx};
use crate::routes::addpeer::AddPeerResponse;
use crate::routes::currstatus::CurrentStatusResponse;

 const ENDOINT_STATUS: &str = "/node/status";

const ENDOINT_SYNC: &str = "/node/sync";
const ENDOINT_SYNC_QUERY_KEY: &str = "begin_hash";

const ENDOINT_ADD_PERR: &str = "/node/peer";
const ENDOINT_ADD_PERR_QUERY_KEY_IP: &str = "ip";

const ENDOINT_ADD_PERR_QUERY_KEY_PORT: &str = "port";
const ENDOINT_ADD_PERR_QUERY_KEY_ACCOUNT: &str = "account";

impl Node {
    pub async fn sync(&self) -> Result<()> {
        info!("syncing with peers,with 25 second interval");
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(25));
        loop {
            select! {
                _ = ticker.tick() => {
                    self.do_sync().await?;
                }
            }

        }
    }




    pub async fn do_sync(&self) -> Result<()> {
        info!("do sync start");
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
            match self.sync_pending_txs(peer,&peer_curr_state.pending_txs).await{
                Ok(_) => {
                    info!("sync txs success");
                }
                Err(e) => {
                    info!("sync txs error: {:?}", e);
                    continue;
                }
            }
        }
        Ok(())
    }

    pub async fn join_know_peers(&self, peer:&PeerNode) -> Result<()> {
        // 通知对方，我是谁，对方把我加入到对方的peer列表中，我也要把对方加入到我的peer列表中
        info!("===begin let peer join me====");
        info!("peer ip: {}, port: {},my ip :{}, port:{}",peer.ip,peer.port,self.info.ip,self.info.port);
        if peer.is_connected{
            return Ok(());
        }
        let url = format!("http://{}:{}{}?{}={}&{}={}&{}={}",
                          peer.ip,
                          peer.port,
                          ENDOINT_ADD_PERR,
                          ENDOINT_ADD_PERR_QUERY_KEY_IP,
                          self.info.ip,
                          ENDOINT_ADD_PERR_QUERY_KEY_PORT,
                          self.info.port,
                          ENDOINT_ADD_PERR_QUERY_KEY_ACCOUNT,
                          self.info.account,
        );
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
        info!("===begin sync blocks from peer: {}====",peer.tcp_addr());
        let bstatus = self.get_status().await;
        let local_block_height =bstatus.get_height();
        let local_block_hash =bstatus.get_last_block_hash();

        // 如果本地高度大于等于对方高度，不需要同步
        //info!("my height:{:?},peer height:{:?}",local_block_height,node_status.height);

        // 如果本地为没有块，对方有至少一块，需要同步，从对方的第一块开始同步，begin_hash=Default::default()
        if local_block_height==0&&local_block_hash==Default::default()&&node_status.hash!=Default::default(){
            info!("local_block_is empty ,async from first block");
            let blocks= fetch_blocks(peer,Default::default()).await?;
            self.add_blocks(blocks).await?;
            return Ok(());
        }

        if local_block_height<node_status.height{
            info!("need async {} block from peer {}",node_status.height-local_block_height,peer.tcp_addr());
            let blocks= fetch_blocks(peer,local_block_hash).await?;
            self.add_blocks(blocks).await?;
            return Ok(());
        }

        Ok(())
    }

    pub async fn sync_known_peers(&self,peer:&PeerNode,node_status:&CurrentStatusResponse) -> Result<()> {
        // 把对方的peer列表同步到本地
        info!("===begin sync known peers from peer: {:?}", peer.tcp_addr());
        for (tcp_addr,peer) in node_status.known_peers.iter(){
            //info!("know peers from peer is : {:?}", peer);

            let mut peerc=peer.clone();

            if !self.is_known_peer(peer).await{
                peerc.is_connected=false; // 从对方获取的peer列表中，我没有的，不一定已经连接上了
                self.add_peer_to_known_peers(peerc.tcp_addr(), peerc.clone()).await;
            }
        }
        Ok(())
    }
    pub async fn sync_pending_txs(&self,peer:&PeerNode,txs:&Vec<Tx>) -> Result<()> {
        // 把对方的pending txs同步到本地
        info!("===begin sync pending txs from peer: {:?}", peer.tcp_addr());
        for tx in txs.iter(){
            //info!("pending tx from peer is : {:?}", tx);
            self.add_pending_tx(tx.clone(),peer).await?;
        }
        Ok(())
    }


    pub async fn add_blocks(&self, block:Vec< Block>) -> Result<()> {
        let mut bstatus = self.bstatus.write().await;
        for b in block.iter() {
            bstatus.add_block(b.clone())?;
            info!("将新同步的block 发送到channel{:?}", b);
            let new_sync_block_channel = self.new_sync_block_channel.clone();
            new_sync_block_channel.send(b.clone()).await?;
        }
        Ok(())
    }
    pub async fn add_block(&self, block:Block) -> Result<()> {
        let mut bstatus = self.bstatus.write().await;
        bstatus.add_block(block)?;
        Ok(())
    }
    pub(crate) async fn is_known_peer(&self, peer:&PeerNode) -> bool {
        // is myself
        if peer.ip==self.info.ip&&peer.port==self.info.port{
            return true;
        }
        let known_peers = self.get_known_peers().await;
        let tcp_addr=peer.tcp_addr();
        known_peers.contains_key(&tcp_addr)
    }
}


pub async fn query_peer_status(peer: &PeerNode) -> Result<CurrentStatusResponse> {
    info!("===begin querying peer status=== from node: {:?}", peer.tcp_addr());
    let tcp_addr = peer.tcp_addr();
    let url = format!("http://{}{}", tcp_addr,ENDOINT_STATUS);
    //info!("querying peer ：url: {:?}", url);
    let client = reqwest::Client::new();
    // 处理掉线情况
    match client.get(&url).send().await {
        Ok(resp) => {
            let resp: CurrentStatusResponse = resp.json().await?;
            info!("query status: know_peeris {:?}", resp.known_peers);
            Ok(resp)
        }
        Err(e) => {
            info!("query peer status error: cant connect peer", );
            Err(anyhow!("connect peer error"))
        }
    }

}
pub async fn fetch_blocks(peer: &PeerNode,begin_hash:BHash) -> Result<Vec<Block>> {
    info!("===middle sync block=== fetching blocks from peer: {:?}", peer.tcp_addr());
    let tcp_addr = peer.tcp_addr();
    let begin_hash_str = hex::encode(begin_hash.0);

    let url = format!("http://{}{}?{}={}",
                      tcp_addr,
                      ENDOINT_SYNC,
                      ENDOINT_SYNC_QUERY_KEY,
                      begin_hash_str);
    //info!("fetching blocks ：url: {:?}", url);

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.unwrap().json::<Vec<Block>>().await?;
    Ok(resp)
}