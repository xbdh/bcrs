use std::collections::HashMap;
use std::sync::Arc;
use axum::{Extension};
use axum::extract::{Json,State};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use parking_lot::RwLock;
//use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::database::{BHash, bstate, TxType};
use crate::node::{Node,PeerNode};

pub async fn curr_status(
    State(node):State<Arc<RwLock<Node>>>
) -> impl IntoResponse {
    //let bstatus =&node.read().bstatus.clone();
    let node = node.read();
    let bstatus = &node.bstatus;

    let cur = CurrentStatusResponse{
        height: bstatus.get_last_block().header.number,
        hash: bstatus.get_last_block_hash(),
        known_peers: node.known_peers.clone(),

    };
    //let  res= serde_json::to_string(&cur).unwrap();
    //(StatusCode::OK, Json(cur))
    Json(cur)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CurrentStatusResponse {
    pub hash :BHash,
    pub height:u64,
    pub known_peers :HashMap<String,PeerNode>,
}