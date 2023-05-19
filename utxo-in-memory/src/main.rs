// #[macro_use]
extern crate lazy_static;

fn main() {
    // init utxo
    // init_utxo();
    let mut keyst = Testmut { key: 3 };
    println!("first:{:#?}", keyst);
    keyst.check();
    println!("updated:{:#?}", keyst);
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
