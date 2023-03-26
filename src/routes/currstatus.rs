use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Json,State};
use axum::response::IntoResponse;

use log::info;

use serde::{Deserialize, Serialize};
use crate::database::{BHash, Tx};
use crate::node::{Node,PeerNode};

pub async fn curr_status(
    State(node):State<Arc<Node>>
) -> impl IntoResponse {
    info!("current status handler");


    let bstatus = node.get_status().await;
    let mut known_peers = node.get_known_peers().await;
    let cur = CurrentStatusResponse{
        height: bstatus.get_last_block().header.number,
        hash: bstatus.get_last_block_hash(),
        known_peers: known_peers,
        pending_txs:Default::default(),
    };
    //info!("current status result: {:?}", cur);
    Json(cur)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CurrentStatusResponse {
    pub hash :BHash,
    pub height:u64,
    pub known_peers :HashMap<String,PeerNode>,
    pub pending_txs:Vec<Tx>,
}