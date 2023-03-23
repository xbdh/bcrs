use std::collections::HashMap;
use std::sync::Arc;
use axum::{Json};

use axum::response::IntoResponse;

use axum::extract::State;
use log::info;
use serde::{Deserialize, Serialize};
use crate::database::tx::Account;
use crate::database::{BHash};
use crate::node::{Node};
pub async fn balances_list(
    State(node):State<Arc<Node>>
) -> impl IntoResponse {
    info!("balances list handler");

    let bstatus = node.bstatus.read().await;

    let bs = bstatus.get_balance();
    let bs_res=BalanceListResponse{
        hash: bstatus.get_last_block_hash(),
        balances: bs,
    };
    Json(bs_res)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BalanceListResponse {
    hash :BHash,
    balances: HashMap<Account,u64>,
}