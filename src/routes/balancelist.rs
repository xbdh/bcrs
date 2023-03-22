use std::collections::HashMap;
use std::sync::Arc;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::extract::State;
use parking_lot::RwLock;
//use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::database::tx::Account;
use crate::database::{BHash, bstate};
use crate::node::{Node,PeerNode};
pub async fn balances_list(
    State(node):State<Arc<RwLock<Node>>>
) -> impl IntoResponse {

    let bstatus =&node.read().bstatus;
    let bs = bstatus.get_balance();
    let bs_res=BalanceListResponse{
        hash: bstatus.get_last_block_hash(),
        balances: bs,
    };
    //let res = serde_json::to_string(&bs_res).unwrap();
    (StatusCode::OK, Json(bs_res))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BalanceListResponse {
    hash :BHash,
    balances: HashMap<Account,u64>,
}