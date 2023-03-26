use axum::{routing::get , Router};
use axum::routing::post;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
// use tokio::sync::RwLock; //不要瞎几把用parking_lot，std 的Rwlock，会线程不安全
use std::io::Write;

use bcrs::routes::{balances_list, curr_status, sync_blocks};
use bcrs::routes::{tx_add};
use bcrs::node::{Node,PeerNode};
use clap::{Parser};
use bcrs::routes::addpeer::add_peer;
use log::{info, LevelFilter};
use env_logger::{Builder, fmt::Color};

#[derive(Parser, Debug, Clone)]
#[clap(name = "http", version = "0.1.0", author = "rain")]
struct Opts {
    #[clap(short, long)]
    pub name: String,
    #[clap(short, long)]
    pub ip: String,
    #[clap(short, long)]
    pub port: u16,
    #[clap(short, long)]
    pub dir: String,
}

#[tokio::main]
async fn main(){
    Builder::new()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            let mut style = buf.style();
            match record.level() {
                log::Level::Trace => style.set_color(Color::Magenta),
                log::Level::Debug => style.set_color(Color::Blue),
                log::Level::Info => style.set_color(Color::Green),
                log::Level::Warn => style.set_color(Color::Yellow),
                log::Level::Error => style.set_color(Color::Red).set_bold(true),
            };
            writeln!(
                buf,
                "{timestamp} [{level}] ({file}:{line}): {message}",
                timestamp =  chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level = style.value(record.level()),
                file = record.file().unwrap_or("unknown"),
                line = record.line().unwrap_or(0),
                message = record.args()
            )
        })
        .init();


    let opts: Opts = Opts::parse();

    let bootstrap_node = PeerNode::new("127.0.0.1".to_string(),3001,true,false);

    let node = Node::new(opts.name,opts.dir.clone(),opts.ip,opts.port,bootstrap_node).unwrap();

    let http_node = Arc::new(node);
    let sync_node = http_node.clone();
    let mine_node = http_node.clone();

    tokio::spawn(async move {
        // sleep for 5 seconds
        tokio::time::sleep(Duration::from_secs(3)).await;
        info!("start sync blocks...")   ;
        let _s= sync_node.sync().await;

    });

    tokio::spawn(async move {
        // sleep for 5 seconds
        tokio::time::sleep(Duration::from_secs(3)).await;
        info! ("start mine blocks...")   ;
        let _s= mine_node.mine().await;

    });

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/balances/list", get(balances_list))
        // `POST /users` goes to `create_user`
        .route("/add/tx", post(tx_add))
        .route("/node/status", get(curr_status))
        .route("/node/sync", get(sync_blocks))
        .route("/node/peer", get(add_peer))
        .with_state(http_node);

    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port.clone()));
    info!("node listening on: {},dir path:{} ", addr,opts.dir);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}