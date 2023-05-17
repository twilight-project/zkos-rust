pub type LogSequence = usize;

pub enum TxInputType {
    Coin = 0,  //uint8
    Memo = 1,  //uint8
    State = 2, //uint8
}
