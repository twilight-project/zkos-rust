use crate::db::KeyId;
use crate::pgsql::{POSTGRESQL_POOL_CONNECTION, THREADPOOL_SQL_QUERY, THREADPOOL_SQL_QUEUE};
use crate::ThreadPool;
use r2d2_postgres::postgres::types::ToSql;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoOutputRaw {
    pub output: Vec<u8>,
    pub height: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoHexDecodeResult {
    pub result: Vec<UtxoOutputRaw>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoHexEncodedResult {
    pub result: Option<String>,
}

impl UtxoHexEncodedResult {
    pub fn encode_to_hex(decoded_data: Vec<UtxoOutputRaw>) -> Self {
        if decoded_data.len() > 0 {
            UtxoHexEncodedResult {
                result: Some(hex::encode(&bincode::serialize(&decoded_data).unwrap())),
            }
        } else {
            UtxoHexEncodedResult { result: None }
        }
    }
}
impl UtxoHexDecodeResult {
    pub fn decode_from_hex(encoded_data: String) -> Self {
        UtxoHexDecodeResult {
            result: bincode::deserialize(&hex::decode(&encoded_data).unwrap()).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUtxoFromDB {
    pub start_block: usize,
    pub end_block: usize,
    pub limit: usize,
    pub pagination: usize,
}

pub fn get_utxo_from_db_by_block_height_range(
    start_block: usize,
    end_block: usize,
    limit: usize,
    pagination: usize,
) -> Result<UtxoHexDecodeResult, std::io::Error> {
    let public_threadpool = THREADPOOL_SQL_QUERY.lock().unwrap();
    let (sender, receiver) = mpsc::channel();
    public_threadpool.execute(move || {

        let query = format!("SELECT utxo, output, owner_address, txid, vout, block_height FROM public.utxo_coin_logs where block_height between {} and {} order by block_height asc OFFSET {} limit {};",start_block,end_block,pagination*limit,limit);
println!("{}",query);
        let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();
        let mut result: Vec<UtxoOutputRaw> = Vec::new();
        match client.query(&query, &[]) {
            Ok(data) => {
                for row in data {
                    result.push(UtxoOutputRaw {
                        output: row.get("output"),
                        height: row.get("block_height"),
                    });
                }
                sender.send(Ok(result)).unwrap();
            }
            Err(arg) => sender
                .send(Err(std::io::Error::new(std::io::ErrorKind::Other, arg)))
                .unwrap(),
        }

    });

    drop(public_threadpool);

    match receiver.recv().unwrap() {
        Ok(value) => {
            return Ok(UtxoHexDecodeResult { result: value });
        }
        Err(arg) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, arg));
        }
    };
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    // cargo test -- --nocapture --test create_psql_table_test --test-threads 1
    #[test]
    fn create_psql_table_test() {
        let result = get_utxo_from_db_by_block_height_range(0, 5, 2, 0);

        let mut file1 = File::create("create_psql_table_test.txt").unwrap();
        file1
            .write_all(&serde_json::to_vec(&result.unwrap()).unwrap())
            .unwrap();
    }
}
