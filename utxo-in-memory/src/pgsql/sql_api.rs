use crate::db::KeyId;
use crate::db::UtxokeyidOutput;
use crate::pgsql::{POSTGRESQL_POOL_CONNECTION, THREADPOOL_SQL_QUERY};
use crate::ThreadPool;
use r2d2_postgres::postgres::types::ToSql;
use serde::{Deserialize, Serialize};
use zkvm::Output;
use std::sync::mpsc;
use zkvm::zkos_types::IOType;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoOutputRaw {
    pub utxo_key:Vec<u8>,
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
    pub start_block: i128,
    pub end_block: i128,
    pub limit: i64,
    pub pagination: i64,
    pub io_type: IOType,
}
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum TestCommandString{
UtxoCoinDbLength,
UtxoMemoDbLength,
UtxoStateDbLength,
TakeSnapshotintoLevelDB,
TakeSnapshotintoPostgreSQL,
TakeBackupFromLevelDB,
TakeBackupFromPostgreSQL,
TransferDataFromLevelDBtoPostgreSQL,
LoadBackupFromLevelDB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCommand {
    pub test_command: TestCommandString,

}

pub fn get_utxo_from_db_by_block_height_range(
    start_block: i128,
    end_block: i128,
    limit: i64,
    pagination: i64,
    io_type: IOType,
) -> Result<UtxoHexDecodeResult, std::io::Error> {
    let public_threadpool = THREADPOOL_SQL_QUERY.lock().unwrap();
    let (sender, receiver) = mpsc::channel();
    public_threadpool.execute(move || {
        let mut query:String="".to_string();

        match io_type{
            IOType::Coin=>{   
                if end_block < 0 {
                query = format!("SELECT utxo, output, block_height FROM public.utxo_coin_logs where block_height >= {} order by block_height asc OFFSET {} limit {};",start_block,pagination*limit,limit);
                
                 println!("{}",query);
           }
   
           else {
                query = format!("SELECT utxo, output, block_height FROM public.utxo_coin_logs where block_height between {} and {} order by block_height asc OFFSET {} limit {};",start_block,end_block,pagination*limit,limit);
               println!("{}",query);
           }
   },
            IOType::Memo=>{   if end_block < 0 {
                query = format!("SELECT utxo, output, block_height FROM public.utxo_memo_logs where block_height >= {} order by block_height asc OFFSET {} limit {};",start_block,pagination*limit,limit);
               println!("{}",query);
           }
   
           else {
                query = format!("SELECT utxo, output, block_height FROM public.utxo_memo_logs where block_height between {} and {} order by block_height asc OFFSET {} limit {};",start_block,end_block,pagination*limit,limit);
               println!("{}",query);
           }
   },
            IOType::State=>{   if end_block < 0 {
                query = format!("SELECT utxo, output, block_height FROM public.utxo_state_logs where block_height >= {} order by block_height asc OFFSET {} limit {};",start_block,pagination*limit,limit);
               println!("{}",query);
           }
   
           else {
                query = format!("SELECT utxo, output, block_height FROM public.utxo_state_logs where block_height between {} and {} order by block_height asc OFFSET {} limit {};",start_block,end_block,pagination*limit,limit);
               println!("{}",query);
           }
   }
        }
    

        let mut client = POSTGRESQL_POOL_CONNECTION.get().unwrap();
        let mut result: Vec<UtxoOutputRaw> = Vec::new();
        match client.query(&query, &[]) {
            Ok(data) => {
                for row in data {
                    result.push(UtxoOutputRaw {
                        utxo_key: row.get("utxo"),
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
        let result = get_utxo_from_db_by_block_height_range(0, 5, 2, 0, IOType::Coin);

        let mut file1 = File::create("create_psql_table_test.txt").unwrap();
        file1
            .write_all(&serde_json::to_vec(&result.unwrap()).unwrap())
            .unwrap();
    }
    #[test]
    fn create_set_genesis_sets_test() {
       crate::blockoperations::set_genesis_sets();
    }
}
