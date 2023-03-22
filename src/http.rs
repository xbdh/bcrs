use axum::{routing::get, http::StatusCode, response::IntoResponse, Json, Router, Extension};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
//use std::sync::RwLock;
use log::info;
use parking_lot::RwLock;
use serde::de::Error;
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

    // tokio::spawn(async move {
    //     shared_data_clone.write().run().await;
    // });

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