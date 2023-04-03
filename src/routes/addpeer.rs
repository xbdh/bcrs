
use std::sync::Arc;
use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;
use log::info;
use serde::{Deserialize, Serialize};
use crate::node::{Node, PeerNode};

pub async fn add_peer(
    State(node):State<Arc<Node>>,
    Query(req): Query<AddPeerRequest>,
) -> impl IntoResponse {
    info!("Handler add peer");
    let ip=req.ip;
    let port=req.port;
    let account=req.account;
    info!("peer info: ip:{}, port:{} account:{}", ip, port, account);
    let peer=PeerNode::new(ip, port, account.clone(),false, true);
    let tcp_addr=peer.tcp_addr();
    node.add_peer_to_known_peers(tcp_addr, peer).await;

    let s=AddPeerResponse{
        success:true,
    };
    Json(s)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AddPeerRequest {
    pub ip: String,
    pub port: u16,
    pub account:String,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct AddPeerResponse {
    pub success: bool,
}