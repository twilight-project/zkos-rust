# ZkOS Architecture

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)

**Status:** experimental ⛏️ – Architecture may evolve before v1.0.

## 🎯 System Overview

ZkOS is a privacy-preserving blockchain infrastructure that implements a UTXO-based state management system with three core state types: **Coins**, **Memos**, and **State**. The system provides confidential transactions, programmable data containers, and smart contract state management through zero-knowledge proofs and advanced cryptographic techniques.

## 🏗️ High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ZkOS Blockchain                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Coins     │  │   Memos     │  │   State     │         │
│  │ (Confidential│  │(Programmable│  │(Smart Contract│         │
│  │   Assets)   │  │   Data)     │  │   State)    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│                    UTXO State Model                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Transaction │  │   ZkVM      │  │ Chain Oracle│         │
│  │   System    │  │ (Verification│  │ (Block Sync)│         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│                    Storage Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ In-Memory   │  │ PostgreSQL  │  │   Snapshots │         │
│  │   Storage   │  │  Persistence│  │   & Recovery│         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

## 📊 State Management

### UTXO Model

ZkOS uses a UTXO (Unspent Transaction Output) model for state management, ensuring:

- **Immutability**: Once created, UTXOs cannot be modified
- **Consistency**: State transitions are atomic and verifiable
- **Efficiency**: Fast verification and state queries
- **Privacy**: Confidential state representation

### State Types

#### 🪙 Coins (Type 0)
Confidential digital assets with ElGamal encryption:

```rust
pub struct OutputCoin {
    pub encrypt: ElGamalCommitment,  // Encrypted value
    pub owner: String,               // Owner's address
}
```

**Features:**
- **ElGamal Encryption**: Confidential value commitments
- **Range Proofs**: Value bounds verification
- **Shuffle Proofs**: Input/output privacy
- **Zero-Knowledge**: Transaction privacy

#### 📝 Memos (Type 1)
Programmable data containers with time-bound access:

```rust
pub struct OutputMemo {
    pub script_address: String,      // Script that can access
    pub owner: String,               // Owner's address
    pub commitment: Commitment,      // Pedersen commitment
    pub data: Option<Vec<ZkvmString>>, // Optional memo data
    pub timebounds: u32,             // Time restrictions
}
```

**Features:**
- **Script-Based Access**: Programmable access control
- **Time Restrictions**: Temporal access control
- **Data Storage**: Flexible data containers
- **Commitment-Based**: Confidential data representation

#### 🏗️ State (Type 2)
Smart contract state with nonce-based versioning:

```rust
pub struct OutputState {
    pub nonce: u32,                  // State version number
    pub script_address: String,      // Contract script
    pub owner: String,               // Owner's address
    pub commitment: Commitment,      // Pedersen commitment
    pub state_variables: Option<Vec<ZkvmString>>, // Contract state
    pub timebounds: u32,             // Time restrictions
}
```

**Features:**
- **Nonce-Based Versioning**: Deterministic state transitions
- **Contract State**: Smart contract data storage
- **State Variables**: Flexible state representation
- **Version Control**: Immutable state history

## 🔐 Privacy Architecture

### Zero-Knowledge Proofs

ZkOS implements multiple zero-knowledge proof systems:

#### Range Proofs
- **Bulletproofs**: Efficient range verification
- **Value Bounds**: Prevent overflow/underflow
- **Confidential Amounts**: Hide transaction values

#### Same-Value Proofs
- **Equality Verification**: Prove commitment equality
- **Cross-Type Transfers**: Coin-to-memo conversions
- **Value Consistency**: Maintain value integrity

#### Shuffle Proofs
- **Input/Output Privacy**: Hide transaction relationships
- **QuisQuis Protocol**: Enhanced privacy through shuffling
- **Anonymity Sets**: Increase privacy guarantees

### Cryptographic Primitives

#### ElGamal Encryption
```rust
// Confidential value commitment
let commitment = gens.commit(value, blinding_factor);
```

#### Pedersen Commitments
```rust
// Binding value representation
let commitment = gens.commit(value, randomness);
```

#### Ristretto255
- **Secure Curve**: State-of-the-art elliptic curve
- **Constant-Time**: Side-channel resistance
- **Efficient Operations**: Optimized for performance

## 🏗️ Component Architecture

### Core Components

#### ZkVM (Zero-Knowledge Virtual Machine)
- **Transaction Verification**: R1CS proof validation
- **Program Execution**: ZkVM script processing
- **Constraint System**: Zero-knowledge proof generation
- **State Validation**: UTXO state verification

#### Transaction System
- **Transfer Transactions**: Confidential asset transfers
- **Script Transactions**: Smart contract execution
- **Message Transactions**: Data operations
- **Proof Generation**: Zero-knowledge proof creation

#### UTXO Storage
- **In-Memory Storage**: High-performance partitioned storage
- **PostgreSQL Persistence**: Reliable state persistence
- **Address Mapping**: Efficient address-to-UTXO queries
- **Snapshot System**: State recovery and backup

#### Chain Oracle
- **Block Subscription**: Real-time blockchain integration
- **Transaction Parsing**: Block and transaction decoding
- **State Updates**: UTXO state synchronization
- **Height Tracking**: Block height management

### Storage Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Storage Layer                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              In-Memory Storage                          │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │ Coin UTXOs  │  │ Memo UTXOs  │  │ State UTXOs │     │ │
│  │  │ (Partition 0)│  │ (Partition 1)│  │ (Partition 2)│     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │ │
│  └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Address Mapping                            │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │ Coin Addr   │  │ Memo Addr   │  │ State Addr  │     │ │
│  │  │ → UTXO IDs  │  │ → UTXO IDs  │  │ → UTXO IDs  │     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │ │
│  └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              PostgreSQL Persistence                    │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │ │
│  │  │ UTXO Table  │  │ Address Table│  │ Snapshot    │     │ │
│  │  │ (All Types) │  │ (Mappings)  │  │ Table       │     │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 🔄 State Transitions

### Transaction Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Input     │───▶│ Transaction │───▶│   Output    │
│   UTXOs     │    │ Processing  │    │   UTXOs     │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Remove    │    │   Verify    │    │    Add      │
│   Spent     │    │   Proofs    │    │   New       │
│   UTXOs     │    │   & State   │    │   UTXOs     │
└─────────────┘    └─────────────┘    └─────────────┘
```

### Block Processing

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Block     │───▶│   Parse     │───▶│   Process   │
│  Received   │    │ Transactions│    │   UTXOs     │
└─────────────┘    └─────────────┘    └─────────────┘
                           │                   │
                           ▼                   ▼
                   ┌─────────────┐    ┌─────────────┐
                   │   Validate  │    │   Update    │
                   │   Proofs    │    │   State     │
                   └─────────────┘    └─────────────┘
                                              │
                                              ▼
                                     ┌─────────────┐
                                     │   Snapshot  │
                                     │   State     │
                                     └─────────────┘
```

## 🔧 Configuration & Deployment

### Environment Configuration

```bash
# Database Configuration
export DATABASE_URL="postgresql://user:password@localhost/zkos_db"

# Chain Oracle Configuration
export NYKS_BLOCK_SUBSCRIBER_URL="http://localhost:1317/"

# API Server Configuration
export RPC_SERVER_PORT=8000
export TELEMETRY_PORT=2500

# Block Height Tracking
export BLOCK_HEIGHT_FILE="height.txt"
```

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ZkOS Network                             │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Node 1    │  │   Node 2    │  │   Node N    │         │
│  │ (UTXO Store)│  │ (UTXO Store)│  │ (UTXO Store)│         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Transaction │  │ Chain Oracle│  │   API       │         │
│  │   API       │  │   Service   │  │   Gateway   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│                    Cosmos Blockchain                        │
└─────────────────────────────────────────────────────────────┘
```

## 📈 Performance & Scalability

### Performance Optimizations

#### Storage Optimizations
- **Partitioned Storage**: UTXOs stored by type for efficient access
- **In-Memory Caching**: Fast access to frequently used data
- **Batch Operations**: Efficient bulk updates
- **Indexed Queries**: Optimized address-to-UTXO lookups

#### Processing Optimizations
- **Thread Pools**: Concurrent block processing
- **Async Operations**: Non-blocking I/O operations
- **Proof Batching**: Efficient zero-knowledge proof verification
- **Memory Management**: Optimized memory usage

### Scalability Features

#### Horizontal Scaling
- **Stateless APIs**: API servers can be scaled horizontally
- **Database Sharding**: UTXO storage can be partitioned
- **Load Balancing**: Multiple nodes for high availability

#### Vertical Scaling
- **Memory Optimization**: Efficient data structures
- **CPU Optimization**: Parallel proof verification
- **Storage Optimization**: Compressed state representation

## 🔍 Monitoring & Observability

### Metrics

#### Prometheus Metrics
- `utxo_coin_count`: Number of coin UTXOs
- `utxo_memo_count`: Number of memo UTXOs
- `utxo_state_count`: Number of state UTXOs
- `block_processing_time`: Block processing latency
- `transaction_throughput`: Transactions per second

#### Telemetry
- **Block Height Tracking**: Current blockchain position
- **State Snapshots**: State recovery points
- **Error Tracking**: System error monitoring
- **Performance Metrics**: System performance monitoring

### Logging

#### Log Levels
- **DEBUG**: Detailed debugging information
- **INFO**: General system information
- **WARN**: Warning messages
- **ERROR**: Error conditions

#### Log Categories
- **Block Processing**: Block and transaction processing
- **State Management**: UTXO state operations
- **Proof Verification**: Zero-knowledge proof operations
- **Database Operations**: Storage operations

## 🛡️ Security Considerations

### Cryptographic Security

#### Key Management
- **Secure Key Generation**: Cryptographically secure random number generation
- **Key Storage**: Secure key storage and management
- **Key Rotation**: Regular key rotation procedures

#### Proof Security
- **Zero-Knowledge**: Complete transaction privacy
- **Soundness**: Proof verification guarantees
- **Completeness**: Valid proofs always verify

### System Security

#### Access Control
- **Authentication**: Secure user authentication
- **Authorization**: Role-based access control
- **Audit Logging**: Comprehensive audit trails

#### Network Security
- **TLS Encryption**: Secure communication channels
- **Rate Limiting**: Protection against abuse
- **DDoS Protection**: Distributed denial-of-service protection

## 🔮 Future Enhancements

### Planned Features

#### Enhanced Privacy
- **Advanced Shuffling**: Improved input/output privacy
- **Ring Signatures**: Enhanced anonymity
- **Mix Networks**: Multi-hop privacy

#### Performance Improvements
- **Proof Aggregation**: Batch proof verification
- **Parallel Processing**: Enhanced concurrency
- **Optimized Storage**: Improved data structures

#### Developer Experience
- **SDK Development**: Comprehensive developer SDK
- **Tooling**: Development and debugging tools
- **Documentation**: Enhanced documentation and examples

### Research Areas

#### Zero-Knowledge Proofs
- **Recursive Proofs**: Efficient proof composition
- **Universal Circuits**: General-purpose zero-knowledge
- **Proof Systems**: New proof system research

#### Blockchain Integration
- **Cross-Chain**: Multi-blockchain support
- **Layer 2**: Layer 2 scaling solutions
- **Interoperability**: Blockchain interoperability

---

**This architecture document is a living document and will be updated as the system evolves.** 