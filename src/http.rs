use axum::{routing::get , Router};
use axum::routing::post;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use log::info;
// use tokio::sync::RwLock; //不要瞎几把用parking_lot，std 的Rwlock，会线程不安全


use bcrs::routes::{balances_list, curr_status, sync_blocks};
use bcrs::routes::{tx_add};
use bcrs::node::{Node,PeerNode};
use clap::{Parser};

#[derive(Parser, Debug, Clone)]
#[clap(name = "http", version = "0.1.0", author = "rain")]
struct Opts {
    #[clap(short, long)]
    pub port: u16,
    #[clap(short, long)]
    pub dir: String,
}

#[tokio::main]
async fn main(){
    env_logger::init();
    let opts: Opts = Opts::parse();

    let bootstrap_node = PeerNode::new("127.0.0.1".to_string(),3001,true,true);

    let node = Node::new(opts.dir.clone(),bootstrap_node).unwrap();

    let shared_data = Arc::new(node);
    let shared_data_clone = shared_data.clone();

    tokio::spawn(async move {
        // sleep for 5 seconds
        tokio::time::sleep(Duration::from_secs(3)).await;
        let _s=shared_data_clone.sync().await;

    });

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/balances/list", get(balances_list))
        // `POST /users` goes to `create_user`
        .route("/add/tx", post(tx_add))
        .route("/node/status", get(curr_status))
        .route("/node/sync", get(sync_blocks))
        .with_state(shared_data);

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port.clone()));
    info!("node listening on: {},dir path:{} ", addr,opts.dir);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}