// use std::str::FromStr;
// use secp256k1::ecdsa::RecoverableSignature;
// use secp256k1::ecdsa::RecoveryId;
// use rlp::Rlp;
// use ethereum_types::{H160, U256};
// use sha3::Keccak256;
// use sha3::Digest;
// use hex::FromHex;
// use secp256k1::Message;
//
// fn main() {
//     let signed_transaction = "..."; // 这里填写已签名的交易数据的16进制字符串表示
//     let sender_address = "742d35Cc6634C0532925a3b844Bc454e4438f44e";
//
//     // 解码已签名的交易数据
//     let signed_tx_bytes = Vec::<u8>::from_hex(signed_transaction).unwrap();
//     let rlp = Rlp::new(&signed_tx_bytes);
//
//     // 提取签名和原始交易数据
//     let v = rlp.at(6).unwrap().as_val::<u64>().unwrap();
//     let r = rlp.at(7).unwrap().as_val::<U256>().unwrap();
//     let s = rlp.at(8).unwrap().as_val::<U256>().unwrap();
//
//     let mut rlp_stream = rlp::RlpStream::new_list(6);
//     for i in 0..6 {
//         rlp_stream.append_raw(rlp.at(i).unwrap().as_raw(), 1);
//     }
//     let raw_transaction = rlp_stream.out();
//
//     // 计算原始交易数据的 Keccak-256 哈希
//     let transaction_hash = Keccak256::digest(&raw_transaction);
//
//     // 从签名中恢复发送者公钥
//     let rec_id = RecoveryId::from_i32((v - 27) as i32).unwrap();
//
//     let mut signature_bytes = [0u8; 64];
//     signature_bytes[..32].copy_from_slice(&r );
//     signature_bytes[32..].copy_from_slice(&s.to_fixed_bytes());
//
//     let signature = RecoverableSignature::from_compact(&signature_bytes/* &[u8] */, rec_id/* secp256k1::ecdsa::RecoveryId */).unwrap();
//     let secp =secp256k1::Secp256k1::new();
//     let public_key = secp.recover_ecdsa(&Message::from_slice(&transaction_hash).unwrap(), &signature).unwrap();
//
//     // 计算发送者地址
//     let mut keccak = Keccak256::new();
//     keccak.update(&public_key.serialize_uncompressed()[1..]);
//     let mut sender_hash = [0u8; 32];
//     keccak.finalize_into(&mut sender_hash);
//     let recovered_address = H160::from_slice(&sender_hash[12..]);
//
//     // 验证交易的合法性
//     if H160::from_str(sender_address).unwrap() == recovered_address {
//         println!("交易是合法的");
//     } else {
//         println!("交易是非法的");
//     }
// }

fn main(){

}