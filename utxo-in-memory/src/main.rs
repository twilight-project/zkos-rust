// #[macro_use]

use utxo_in_memory::utxo_set;
extern crate lazy_static;
use utxo_in_memory::*;

fn main() {
    // init utxo
    // init_utxo();
    let mut keyst = Testmut { key: 3 };
    println!("first:{:#?}", keyst);
    keyst.check();
    println!("updated:{:#?}", keyst);

    // println!("{:#?}", utxo_set::load_genesis_sets().len());
}

#[derive(Debug, Clone)]
pub struct Testmut {
    key: usize,
}

impl Testmut {
    pub fn check(&mut self) {
        self.key = 5;
    }
}
