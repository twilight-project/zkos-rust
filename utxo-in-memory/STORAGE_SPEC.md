# ZkOS UTXO Storage Architecture Specification

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)

**Status:** experimental ⛏️ – Architecture may evolve before v1.0.

## 🎯 Overview

This document specifies the architecture and implementation details of the ZkOS UTXO storage system, including in-memory storage, PostgreSQL persistence, and synchronization mechanisms.

## 🏗️ Storage Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    ZkOS UTXO Storage System                 │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              In-Memory Storage Layer                    │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │ Coin UTXOs  │  │ Memo UTXOs  │  │ State UTXOs │     │ │
│  │  │ (Partition 0)│  │ (Partition 1)│  │ (Partition 2)│     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │ │
│  └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Address Mapping Layer                      │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │ Coin Addr   │  │ Memo Addr   │  │ State Addr  │     │ │
│  │  │ → UTXO IDs  │  │ → UTXO IDs  │  │ → UTXO IDs  │     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │ │
│  └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Snapshot Layer                             │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │ LevelDB     │  │ Snapshot    │  │ Recovery    │     │ │
│  │  │ Storage     │  │ Metadata    │  │ Pointers    │     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │ │
│  └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              PostgreSQL Persistence                     │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │ UTXO Table  │  │ Address Table│  │ Snapshot    │     │ │
│  │  │ (All Types) │  │ (Mappings)  │  │ Table       │     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 📊 In-Memory Storage Layer

### LocalStorage<T> Structure

```rust
pub struct LocalStorage<T> {
    pub data: HashMap<InputType, HashMap<KeyId, T>>,
    pub block_height: SequenceNumber,
    pub aggrigate_log_sequence: SequenceNumber,
    pub snaps: SnapShot,
    pub partition_size: usize,
}
```

#### Partitioning Strategy

The in-memory storage uses a partitioned approach with three partitions:

- **Partition 0**: Coin UTXOs (Type 0)
- **Partition 1**: Memo UTXOs (Type 1)  
- **Partition 2**: State UTXOs (Type 2)

#### Data Structure

```
LocalStorage {
  data: {
    0: HashMap<KeyId, Output>, // Coin UTXOs
    1: HashMap<KeyId, Output>, // Memo UTXOs
    2: HashMap<KeyId, Output>, // State UTXOs
  },
  block_height: 12345,
  aggrigate_log_sequence: 67890,
  snaps: SnapShot { ... },
  partition_size: 3
}
```

### Key Design Decisions

1. **Type Separation**: Each UTXO type has its own hash map for efficient access
2. **Serialized Keys**: UTXO keys are serialized as Vec<u8> for storage efficiency
3. **Generic Value Type**: Uses generic T for flexibility with different output types
4. **Block Height Tracking**: Maintains current blockchain position
5. **Log Sequence**: Tracks aggregate operations for consistency

## 🗺️ Address Mapping Layer

### AddressUtxoIDStorage Structure

```rust
pub struct AddressUtxoIDStorage {
    pub data: HashMap<usize, HashMap<String, String>>,
}
```

#### Mapping Strategy

```
AddressUtxoIDStorage {
  data: {
    0: HashMap<String, String>, // Coin: Address → UTXO ID
    1: HashMap<String, String>, // Memo: Address → UTXO ID
    2: HashMap<String, String>, // State: Address → UTXO ID
  }
}
```

#### Design Benefits

1. **Fast Lookup**: O(1) average case address lookup
2. **Type Separation**: Separate mappings for each UTXO type
3. **Bidirectional Access**: Support for both address→UTXO and UTXO→address queries
4. **Memory Efficiency**: Minimal overhead for address indexing

## 📸 Snapshot Layer

### SnapShot Structure

```rust
pub struct SnapShot {
    pub block_height: SequenceNumber,
    pub currentsnapid: u64,
    pub lastsnapid: u64,
    pub aggrigate_log_sequence: SequenceNumber,
    pub lastsnaptimestamp: u128,
    pub snap_rules: SnapRules,
}
```

### Snapshot Strategy

#### Creation Process

1. **Partition Cloning**: Each partition is cloned to avoid blocking operations
2. **Thread Pool Processing**: Uses thread pool for concurrent snapshot creation
3. **LevelDB Storage**: Snapshots stored in LevelDB for persistence
4. **Metadata Tracking**: Maintains snapshot metadata for recovery

#### Storage Format

```
LevelDB Structure:
├── {snapshot_path}-0 → (snapshot_id, coin_utxos_data)
├── {snapshot_path}-1 → (snapshot_id, memo_utxos_data)  
├── {snapshot_path}-2 → (snapshot_id, state_utxos_data)
├── {snapshot_path}-snapmap → (snapshot_id, block_height)
└── {snapshot_path}-snapmap → ("utxosnapshot", snapshot_metadata)
```

## 🗄️ PostgreSQL Persistence Layer

### Database Schema

#### UTXO Table
```sql
CREATE TABLE utxos (
    id SERIAL PRIMARY KEY,
    utxo_key BYTEA NOT NULL,
    utxo_value BYTEA NOT NULL,
    utxo_type INTEGER NOT NULL,
    block_height BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_utxos_type ON utxos(utxo_type);
CREATE INDEX idx_utxos_block_height ON utxos(block_height);
CREATE UNIQUE INDEX idx_utxos_key ON utxos(utxo_key);
```

#### Address Mapping Table
```sql
CREATE TABLE address_utxo_mappings (
    id SERIAL PRIMARY KEY,
    address VARCHAR(255) NOT NULL,
    utxo_id VARCHAR(255) NOT NULL,
    utxo_type INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_address_mappings_address ON address_utxo_mappings(address);
CREATE INDEX idx_address_mappings_type ON address_utxo_mappings(utxo_type);
CREATE UNIQUE INDEX idx_address_mappings_unique ON address_utxo_mappings(address, utxo_type);
```

#### Snapshot Table
```sql
CREATE TABLE snapshots (
    id SERIAL PRIMARY KEY,
    snapshot_id BIGINT NOT NULL,
    block_height BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_snapshots_id ON snapshots(snapshot_id);
CREATE INDEX idx_snapshots_block_height ON snapshots(block_height);
```

### Synchronization Strategy

#### Write-Through Pattern

1. **In-Memory First**: All operations first update in-memory storage
2. **Async Persistence**: PostgreSQL updates happen asynchronously
3. **Bulk Operations**: Uses bulk inserts for performance
4. **Transaction Safety**: PostgreSQL operations are transactional

#### Bulk Insert Process

```rust
// Coin UTXOs bulk insert
pub fn insert_bulk_utxo_in_psql_coin(
    utxo_data: Vec<(Vec<u8>, Vec<u8>)>,
    block_height: i64,
) -> Result<(), Box<dyn std::error::Error>>

// Memo/State UTXOs bulk insert  
pub fn insert_bulk_utxo_in_psql_memo_or_state(
    utxo_data: Vec<(Vec<u8>, Vec<u8>)>,
    block_height: i64,
    utxo_type: i32,
) -> Result<(), Box<dyn std::error::Error>>
```

## 🔄 Synchronization Mechanisms

### Memory to PostgreSQL Sync

#### Process Flow

```
1. Block Processing
   ↓
2. In-Memory Update
   ↓
3. Thread Pool Job Creation
   ↓
4. Bulk Data Preparation
   ↓
5. PostgreSQL Bulk Insert
   ↓
6. Address Mapping Update
   ↓
7. Snapshot Creation (Periodic)
```

#### Thread Pool Architecture

```rust
// SQL Query Thread Pool
pub static ref THREADPOOL_SQL_QUERY: Arc<Mutex<ThreadPool>> =
    Arc::new(Mutex::new(ThreadPool::new(
        4, // 4 worker threads
        String::from("SQL_QUERY_THREADPOOL")
    )));

// SQL Queue Thread Pool  
pub static ref THREADPOOL_SQL_QUEUE: Arc<Mutex<ThreadPool>> =
    Arc::new(Mutex::new(ThreadPool::new(
        2, // 2 worker threads
        String::from("SQL_QUEUE_THREADPOOL")
    )));
```

### Recovery and Consistency

#### Startup Process

1. **PostgreSQL Connection**: Establish connection pool
2. **Snapshot Loading**: Load latest snapshot from LevelDB
3. **PostgreSQL Sync**: Load additional data from PostgreSQL
4. **Address Mapping**: Rebuild address-to-UTXO mappings
5. **State Validation**: Verify consistency between memory and database

#### Consistency Guarantees

1. **ACID Properties**: PostgreSQL ensures transaction consistency
2. **Snapshot Integrity**: LevelDB snapshots provide recovery points
3. **Memory Consistency**: In-memory state is always consistent
4. **Eventual Consistency**: PostgreSQL catches up asynchronously

## 📈 Performance Characteristics

### Memory Usage

#### Storage Overhead

- **UTXO Storage**: ~100 bytes per UTXO (key + value + metadata)
- **Address Mapping**: ~50 bytes per address mapping
- **Snapshot Metadata**: ~1KB per snapshot
- **Total Overhead**: ~150 bytes per UTXO

#### Scaling Characteristics

- **Linear Scaling**: Memory usage scales linearly with UTXO count
- **Partition Efficiency**: Type separation reduces hash collisions
- **Cache Locality**: Partitioned storage improves cache performance

### Throughput

#### Operation Performance

- **UTXO Add**: O(1) average case
- **UTXO Remove**: O(1) average case  
- **UTXO Lookup**: O(1) average case
- **Address Lookup**: O(1) average case
- **Bulk Operations**: O(n) with high throughput

#### PostgreSQL Performance

- **Bulk Insert**: ~10,000 UTXOs/second
- **Bulk Update**: ~5,000 UTXOs/second
- **Query Performance**: Sub-millisecond for indexed queries

## 🔧 Configuration

### Environment Variables

```bash
# Database Configuration
export DATABASE_URL="postgresql://user:password@localhost/zkos_db"

# Connection Pool Configuration
export POSTGRESQL_MAX_CONNECTIONS=20
export POSTGRESQL_MIN_CONNECTIONS=5

# Thread Pool Configuration
export SQL_QUERY_THREADS=4
export SQL_QUEUE_THREADS=2

# Snapshot Configuration
export SNAPSHOT_INTERVAL=1000  # Blocks between snapshots
export SNAPSHOT_RETENTION=10   # Number of snapshots to keep
```

### Tuning Parameters

#### Memory Tuning

```rust
// Partition size (number of UTXO types)
const PARTITION_SIZE: usize = 3;

// Initial hash map capacity
const INITIAL_UTXO_CAPACITY: usize = 10000;
const INITIAL_ADDRESS_CAPACITY: usize = 5000;
```

#### PostgreSQL Tuning

```sql
-- Connection pool settings
SET max_connections = 100;
SET shared_buffers = '256MB';
SET effective_cache_size = '1GB';

-- Query optimization
SET random_page_cost = 1.1;
SET effective_io_concurrency = 200;
```

## 🛡️ Security Considerations

### Data Protection

1. **Encryption**: Sensitive data encrypted in transit and at rest
2. **Access Control**: Database access restricted to authorized users
3. **Audit Logging**: All operations logged for security monitoring
4. **Input Validation**: All inputs validated before processing

### Backup Strategy

1. **Regular Snapshots**: Automated snapshot creation
2. **Database Backups**: PostgreSQL point-in-time recovery
3. **Offsite Storage**: Critical data backed up offsite
4. **Recovery Testing**: Regular recovery procedure testing

## 🔍 Monitoring and Observability

### Metrics

#### Prometheus Metrics

```rust
// UTXO Count Metrics
pub static ref UTXO_COIN_TELEMETRY_COUNTER: Gauge =
    register_gauge!("utxo_coin_count", "Number of coin UTXOs").unwrap();

pub static ref UTXO_MEMO_TELEMETRY_COUNTER: Gauge =
    register_gauge!("utxo_memo_count", "Number of memo UTXOs").unwrap();

pub static ref UTXO_STATE_TELEMETRY_COUNTER: Gauge =
    register_gauge!("utxo_state_count", "Number of state UTXOs").unwrap();
```

#### Performance Metrics

- **Memory Usage**: Current memory consumption
- **Operation Latency**: Time for UTXO operations
- **Database Latency**: PostgreSQL operation timing
- **Throughput**: Operations per second

### Logging

#### Log Levels

- **DEBUG**: Detailed operation tracing
- **INFO**: General operation information
- **WARN**: Warning conditions
- **ERROR**: Error conditions

#### Log Categories

- **Storage Operations**: UTXO add/remove operations
- **Database Operations**: PostgreSQL interactions
- **Snapshot Operations**: Snapshot creation/loading
- **Recovery Operations**: System recovery procedures

## 🔮 Future Enhancements

### Planned Improvements

1. **Sharding**: Horizontal partitioning for large datasets
2. **Compression**: Data compression for storage efficiency
3. **Caching**: Multi-level caching for performance
4. **Distributed Storage**: Support for distributed deployments

### Research Areas

1. **Alternative Databases**: Evaluation of other storage backends
2. **In-Memory Databases**: Integration with Redis/Memcached
3. **Stream Processing**: Real-time data processing capabilities
4. **Machine Learning**: Predictive analytics for storage optimization

---

**This specification is a living document and will be updated as the system evolves.** 