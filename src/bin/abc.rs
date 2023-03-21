use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    hash: String,
    data: String,
}

struct Status {
    db_file: File,
    balances: HashMap<String, i32>,
}

impl Status {
    fn apply_block(&mut self, block: Block) {
        // Apply block to balances
        let amount = block.data.parse::<i32>().unwrap();
        *self.balances.entry(block.hash).or_insert(0) += amount;
    }
}

fn main() {
    let file = File::open("blocks.json").unwrap();
    let bufreader = BufReader::new(&file);

    // Initialize status with balances
    let balances = HashMap::new();
    let mut status = Status { db_file: file, balances };

    // Read blocks and apply to status
    for line in bufreader.lines() {
        let line = line.unwrap();
        let block: Block = serde_json::from_str(&line).unwrap();
        status.apply_block(block);
    }

    println!("{:?}", status.balances);
}