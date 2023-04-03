use std::sync::Arc;
use axum::extract::{Json,State};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use log::info;
use serde::{Deserialize, Serialize};
use crate::database::{Pure, Tx, TxType};
use crate::node::{Node};
use secp256k1::ecdsa::{SerializedSignature, Signature};
use secp256k1::{Secp256k1, Message, SecretKey,PublicKey};
use serde_json::json_internal_vec;
use sha3::{Digest, Keccak256};
use secp256k1::hashes::sha256;

pub async fn tx_add(
    State(node):State<Arc<Node>>,
    Json(req): Json<TxAddRequest>, //顺序很重要，先提取Extension，再提取Json，奇怪的是，如果先提取Json，再提取Extension，就会报错
) -> impl IntoResponse {
    info!("Handler add tx");
    let from=req.from.clone();
    let to=req.to.clone();
    let value=req.value.clone();
    let signature=req.signature.clone();
    let message=req.message.clone();

    let sig=Signature::from_compact(&signature).unwrap();
    let message=Message::from_slice(&message).unwrap();
    let public_key=PublicKey::from_slice(&req.public_key).unwrap();
    println!("public_key:{:?}",public_key);
    println!("signature:{:?}",signature);
    println!("message:{:?}",message);

    let msg=Pure::new(from.clone(),to.clone(),value.clone());
    let msgs=serde_json::to_string(&msg).unwrap();
    let message2 = Message::from_hashed_data::<sha256::Hash>(msgs.as_bytes());
    // from 去掉开头 0x 就是真正地址
    let address=hex::decode(&from[2..]).unwrap().to_vec();

    let address2=public_key_to_eth_address(&public_key);

    info!("address:{:?}",address);
    info!("address2:{:?}",address2);
    if address!=address2.to_vec(){
        info!("地址不匹配");
        return StatusCode::BAD_REQUEST;
    }

    let secp = Secp256k1::new();
    let res=secp.verify_ecdsa(&message2, &sig, &public_key);
    if res.is_err(){
        info!("验证失败");
        //info!("Handler add tx error");
        return StatusCode::BAD_REQUEST;
    }
    info!("验证成功");
    let tx=Tx::new(req.from,req.to,req.value,TxType::Normal);
    node.add_tx_to_pending_txs(tx).await;

    StatusCode::OK
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxAddRequest {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub signature: Vec<u8>,
    pub message:Vec<u8>,
    pub public_key:Vec<u8>,
    //pub message:,
    //pub tx_tyep:Option<TxType>,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxAddResponse {
    pub is_success: bool  ,
}

impl TxAddRequest {
    pub fn new(from: String, to: String, value: u64, signature: Vec<u8>, message: Vec<u8>, public_key: Vec<u8>) -> Self {
        Self {
            from,
            to,
            value,
            signature,
            message,
            public_key
        }
    }
}

fn public_key_to_eth_address(&public_key: &PublicKey) -> [u8; 20] {
    //let public_key = PublicKey::from_secret_key(&private_key);
    let serialized_public_key = public_key.serialize_uncompressed();

    let mut keccak = Keccak256::new();

    let mut hash = [0u8; 32];
    keccak.update(&serialized_public_key[1..]);
    hash.copy_from_slice(&keccak.finalize());

    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..]);
    address
}