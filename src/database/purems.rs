use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pure{
    from:String,
    to :String,
    value:u64,
}

impl Pure{
    pub fn new(from: String, to: String, value: u64) -> Self {
        Self {
            from,
            to,
            value,
        }
    }
    
}

struct PureWithSignature{
    from:String,
    to :String,
    value:u64,
    signature:Vec<u8>,
}