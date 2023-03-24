
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
    info!("add peer handler");
    let ip=req.ip;
    let port=req.port;
    info!("add peer ip:{}, port:{}", ip, port);
    let peer=PeerNode::new(ip, port, false, true);
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

}

#[derive(Deserialize, Serialize, Debug)]
pub struct AddPeerResponse {
    pub success: bool,
}