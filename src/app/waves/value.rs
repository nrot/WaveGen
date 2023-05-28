use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BitValue{
    size: usize,
    data: Vec<u8>,
    lsb: bool,
}

impl BitValue{
    pub fn new(size: usize)->Self{
        BitValue { size, data: Vec::with_capacity(size), lsb: true }
    }


}