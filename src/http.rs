use axum::{routing::get, http::StatusCode, response::IntoResponse, Json, Router, Extension};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use log::info;
use tokio::sync::RwLock; //不要瞎几把用parking_lot，std 的Rwlock，会线程不安全
use serde::de::Error;
use tokio::select;
use bcrs::database;
use bcrs::database::BStatus;
use bcrs::routes::{balances_list, curr_status};
use bcrs::routes::{tx_add};
use bcrs::node::{Node,PeerNode};


#[tokio::main]
async fn main(){
    env_logger::init();

    let db_status = database::BStatus::new().unwrap();
    let bootstrap_node = PeerNode::new("127.0.0.1".to_string(),3000,true,true);
    let node = Node::new(3000,db_status,bootstrap_node).unwrap();
    let shared_data = Arc::new(RwLock::new(node));
    let shared_data_clone = shared_data.clone();

    tokio::spawn(async move {
        let mut node = shared_data_clone.write().await;
        // let res=node.sync().await;
        // println!("sync res:{:?}",res);

        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(45));
        loop {
            select! {
                _ = ticker.tick() => {
                    node.fetch_new_block_and_peer().await.unwrap();
                }
            }

        }
    });

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/balances/list", get(balances_list))
        // `POST /users` goes to `create_user`
        .route("/add/tx", post(tx_add))
        .route("/node/status", get(curr_status))
        .with_state(shared_data);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}