use std::sync::Arc;
use axum::extract::{Json,State};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use log::info;
use serde::{Deserialize, Serialize};
use crate::database::{Tx, TxType};
use crate::node::{Node};

pub async fn tx_add(
    State(node):State<Arc<Node>>,
    Json(req): Json<TxAddRequest>, //顺序很重要，先提取Extension，再提取Json，奇怪的是，如果先提取Json，再提取Extension，就会报错
) -> impl IntoResponse {
    info!("tx add handler");
    let tx=Tx::new(req.from,req.to,req.value,TxType::Normal);
    node.add_tx_to_pending_txs(tx).await;

    StatusCode::OK
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxAddRequest {
    pub from: String,
    pub to: String,
    pub value: u64,
    //pub tx_tyep:Option<TxType>,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxAddResponse {
    pub is_success: bool  ,
}