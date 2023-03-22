use std::sync::Arc;
use axum::{Extension};
use axum::extract::{Json,State};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use parking_lot::RwLock;
//use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::database::{bstate, TxType};
use crate::node::{Node,PeerNode};

pub async fn tx_add(
    State(node):State<Arc<RwLock<Node>>>,
    Json(req): Json<TxAddRequest>, //顺序很重要，先提取Extension，再提取Json，奇怪的是，如果先提取Json，再提取Extension，就会报错
) -> impl IntoResponse {

    StatusCode::OK
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxAddRequest {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub tx_tyep:TxType,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxAddResponse {
    pub tx_hash: String,
}