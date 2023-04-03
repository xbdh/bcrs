use std::hash::Hash;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, Message, SecretKey,PublicKey};

use secp256k1::hashes::sha256;
use sha3::{Digest, Keccak256};
use eth_keystore::{decrypt_key, encrypt_key};
use rand::RngCore;
use std::path::Path;
use secp256k1::ecdsa::Signature;
use bcrs::database::Pure;
use bcrs::routes::txadd::TxAddRequest;

fn main()->Result<(), ureq::Error>{
    let secp = Secp256k1::new();
    // let (mut secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    // let message = Message::from_hashed_data::<sha256::Hash>("Hello World!".as_bytes());
    // let sig = secp.sign_ecdsa(&message, &secret_key);
    // assert!(secp.verify_ecdsa(&message, &sig, &public_key).is_ok());
    // let address = public_key_to_eth_address(&public_key);
    //
    // //convert address to hex string with 0x prefix
    // let address_hex = hex::encode(address);
    // let address_hex = format!("0x{}",address_hex.to_uppercase());
    // println!("address:{}",address_hex);
    // let dir = Path::new("./db");
    //
    //
    // let private_key= secret_key.as_ref();
    //
    //
    // let name = encrypt_key(&dir, &mut OsRng, &private_key, "password_to_keystore", Some(&address_hex)).unwrap();
    // println!("name:{}",name);

    let pp=decrypt_key("./db/0x446E89D661D607868FBD8E881E6A15C3797AF140", "password_to_keystore").unwrap();
    let serect_key=SecretKey::from_slice(&pp).unwrap();
    let public_key=PublicKey::from_secret_key(&secp,&serect_key);
    let msg=Pure::new("0x446E89D661D607868FBD8E881E6A15C3797AF140".to_string(),
    "0xE4DFEB6011C12A097190BBFBFF859979E2616AD0".to_string(),
        5000,
    );
    let msgs=serde_json::to_string(&msg).unwrap();
    let message = Message::from_hashed_data::<sha256::Hash>(msgs.as_bytes());
    let sig:Signature = secp.sign_ecdsa(&message.into(), &serect_key);
    println!("sig:{:?}",sig);
    println!("public:{:?}",public_key);
    println!("message:{:?}",message);

    let txreq=TxAddRequest::new(
        "0x446E89D661D607868FBD8E881E6A15C3797AF140".to_string(),
        "0xE4DFEB6011C12A097190BBFBFF859979E2616AD0".to_string(),
        5000,
        sig.serialize_compact().to_vec(),
        message.as_ref().to_vec(),
        public_key.serialize_uncompressed().to_vec(),

    );

    let resp: String = ureq::post("http://127.0.0.1:3001/add/tx")
        .set("X-My-Header", "Secret")
        .send_json(ureq::json!(txreq))?
        .into_string()?;
    Ok(()   )
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

fn private_key_to_eth_address(serect_key: &SecretKey) -> [u8; 20] {
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, serect_key);
    let serialized_public_key = public_key.serialize_uncompressed();

    let mut keccak = Keccak256::new();

    let mut hash = [0u8; 32];
    keccak.update(&serialized_public_key[1..]);
    hash.copy_from_slice(&keccak.finalize());

    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..]);
    address
}
// 将地址转换为公钥
fn address_to_public_key(address: &[u8; 20]) -> PublicKey {
    let mut keccak = Keccak256::new();
    let mut hash = [0u8; 32];
    keccak.update(&address);
    hash.copy_from_slice(&keccak.finalize());
    let mut public_key = [0u8; 65];
    public_key[0] = 4;
    public_key[1..].copy_from_slice(&hash[12..]);
    PublicKey::from_slice(&public_key).unwrap()
}





struct MyPrivateKey(SecretKey);
impl MyPrivateKey{
    fn new(s:SecretKey) -> Self {
        //let secp = Secp256k1::new();
        //let (secret_key, _) = secp.generate_keypair(&mut OsRng);
        Self(s)
    }
}
impl AsRef<[u8]> for MyPrivateKey{
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

// /// Converts a K256 SigningKey to an Ethereum Address
// pub fn address_from_pk<S>(pk: S) ->[u8;20]
//     where
//         S: AsRef<[u8]>,
// {
//     let secret_key = SigningKey::from_bytes(pk.as_ref()).unwrap();
//     let public_key = PublicKey::from(&secret_key.verifying_key());
//     let public_key = public_key.to_encoded_point(/* compress = */ false);
//     let public_key = public_key.as_bytes();
//     debug_assert_eq!(public_key[0], 0x04);
//     let hash = keccak256(&public_key[1..]);
//     let address = hash[12..].to_vec();
//     address.try_into().unwrap()
//     //Ok(Address::from_slice(&hash[12..]))
// }
//
// /// Compute the Keccak-256 hash of input bytes.
// fn keccak256<S>(bytes: S) -> [u8; 32]
//     where
//         S: AsRef<[u8]>,
// {
//     let mut hasher = Keccak256::new();
//     hasher.update(bytes.as_ref());
//     hasher.finalize().into()
// }