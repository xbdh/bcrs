use std::sync::Arc;
use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;
use log::info;
use serde::{Deserialize, Serialize};
use crate::database::{BHash};
use crate::database::block::get_blocks_after_hash;
use crate::node::{Node};

pub async fn sync_blocks(
    Query(q):Query<SyncBlocksQuery>,
    State(node):State<Arc<Node>>
) -> impl IntoResponse {
    info!("sync blocks handler");
    let dir=node.dir_path.clone();
    let bhash=q.from_block;
    let blocks=get_blocks_after_hash(bhash, dir).unwrap();

    Json(blocks)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SyncBlocksQuery {
    pub from_block: BHash,

}