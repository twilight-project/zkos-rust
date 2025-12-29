//! JSON-RPC ID management for request identification.
//!
//! This module provides the `Id` enum for handling JSON-RPC request identifiers,
//! supporting numerical, string, and null ID types as per the JSON-RPC 2.0 specification.
//! Includes utility functions for generating UUID-based IDs and comprehensive
//! serialization testing.

use serde::{Deserialize, Serialize};

use super::utils::uuid_str;

/// JSON-RPC request identifier supporting multiple ID types.
///
/// According to JSON-RPC 2.0 specification, an identifier can be a string,
/// number, or null value. This enum handles all three cases.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(untagged)]
pub enum Id {
    /// Numerical JSON-RPC ID (e.g., 42, 0, -1)
    Num(i64),
    /// String JSON-RPC ID (e.g., "request-1", UUID strings)
    Str(String),
    /// Null JSON-RPC ID (represents absence of identifier)
    None,
}

impl Id {
    /// Creates a new JSON-RPC ID with a random UUID v4 string.
    ///
    /// # Returns
    /// An `Id::Str` containing a UUID v4 string for unique request identification.
    ///
    pub fn uuid_v4() -> Self {
        Self::Str(uuid_str())
    }
}

// impl fmt::Display for Id {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Id::Num(i) => write!(f, "{i}"),
//             Id::Str(s) => write!(f, "{s}"),
//             Id::None => write!(f, ""),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use serde::{de::DeserializeOwned, Serialize};

    use super::*;

    /// Tests JSON-RPC ID serialization and deserialization round-trips.
    ///
    /// Verifies that all ID types (string, number, null) can be properly
    /// serialized to JSON and deserialized back to their original form.
    #[test]
    fn round_tripping_jsonrpc_id() {
        let str = r#""42""#;
        serialization_roundtrip::<Id>(str);

        let str2 = r#""936DA01F-9ABD-4D9D-80C7-02AF85C822A8""#;
        serialization_roundtrip::<Id>(str2);

        let num = r#"42"#;
        serialization_roundtrip::<Id>(num);

        let zero = r#"0"#;
        serialization_roundtrip::<Id>(zero);

        let null = r#"null"#;
        serialization_roundtrip::<Id>(null);
    }

    /// Helper function to test serialization round-trips for any serializable type.
    ///
    /// # Arguments
    /// * `json_data` - JSON string to deserialize and then re-serialize
    ///
    /// # Generic Parameters
    /// * `T` - Type that implements Debug, PartialEq, Serialize, and DeserializeOwned
    fn serialization_roundtrip<T>(json_data: &str)
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let parsed0 = serde_json::from_str::<T>(json_data);
        assert!(parsed0.is_ok());
        let parsed0 = parsed0.unwrap();

        let serialized = serde_json::to_string(&parsed0);
        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();

        let parsed1 = serde_json::from_str::<T>(&serialized);
        assert!(parsed1.is_ok());
        let parsed1 = parsed1.unwrap();

        assert_eq!(parsed0, parsed1);
    }
}
