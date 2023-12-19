use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use utxo_in_memory::db::{LocalDBtrait, LocalStorage};
use utxo_in_memory::{db::LocalDBtrait, UTXO_STORAGE};
fn utxo_store_benchmark(c: &mut Criterion) {
    let mut utxo_storage = UTXO_STORAGE.lock().unwrap();

    let mut arr = black_box([6, 2, 4, 1, 9, -2, 5]);
    let test_transaction = utxo_in_memory::pgsql::deserialize_tx_string();
    let tx_output = test_transaction.get_tx_outputs();
    c.bench_function("utxo_storage", |b| {
        b.iter(|| utxo_storage.add(bincode::serialize(&arr).unwrap(), tx_output[0].clone(), 0))
    });

    utxo_storage.add(bincode::serialize(&arr).unwrap(), tx_output[0].clone(), 0);
    let mut arr1 = black_box([6, 2, 4, 1, 9, -2, 5]);
    c.bench_function("utxo_storage-remove", |b| {
        b.iter(|| utxo_storage.remove(bincode::serialize(&arr1).unwrap(), 0))
    });

    let mut arr2 = black_box([6, 2, 4, 1, 9, -2, 5]);
    c.bench_function("utxo_storage-search", |b| {
        b.iter(|| utxo_storage.search_key(&bincode::serialize(&arr2).unwrap(), 0))
    });
}

criterion_group!(benches, utxo_store_benchmark);
criterion_main!(benches);
