
use bcrs::database::{BHash, Block, Status, Tx, TxType};
fn main() {
    env_logger::init();
    let mut status=Status::new().unwrap();
    let bs=status.get_balance();

    println!("{:?}", bs);
    let tx1=Tx::new("andrej".to_string(), "bob".to_string(), 300, TxType::Normal);
    let tx2=Tx::new("andrej".to_string(), "bob".to_string(), 500, TxType::Normal);
    let tx3=Tx::new("andrej".to_string(), "andrej".to_string(), 100, TxType::Reward);
    let txs=vec![tx1,tx2,tx3];
    let block=Block::new(BHash([0;32]), 0, txs);

    status.add_block(block).unwrap();
    let bs=status.get_balance();
    let h=status.persist().unwrap();

    println!("{:?}", bs);
    println!("{:?}", serde_json::to_string(&h).unwrap());

    let tx4=Tx::new("andrej".to_string(), "bob".to_string(), 800, TxType::Normal);
    let tx5=Tx::new("andrej".to_string(), "cat".to_string(), 10000, TxType::Normal);
    let tx6=Tx::new("andrej".to_string(), "andrej".to_string(), 100, TxType::Reward);
    let txs=vec![tx4,tx5,tx6];
    let block=Block::new(BHash([0;32]), 0, txs);

    status.add_block(block).unwrap();
    let bs=status.get_balance();
    let h=status.persist().unwrap();
    println!("{:?}", serde_json::to_string(&h).unwrap());
    println!("{:?}", bs);


}
