//! Schema definitions for Madara's RocksDB column families.
//!
//! This crate provides comprehensive documentation of all RocksDB column families
//! used by Madara, including their key/value structures, encoding formats, and
//! relationships to other column families.
//!
//! # Overview
//!
//! Madara uses RocksDB with multiple column families organized into categories:
//!
//! - **Blocks**: Block headers, hashes, state diffs, transactions, and receipts
//! - **Contracts**: Contract state including storage, nonces, and class hashes
//! - **Classes**: Cairo class definitions and compiled code
//! - **Tries**: Bonsai Merkle Patricia Tries for state commitment
//! - **Meta**: Node metadata and configuration
//! - **Messaging**: L1 to L2 message handling
//! - **Mempool**: Pending transaction storage
//! - **Events**: Event bloom filters for efficient querying
//!
//! # Usage
//!
//! ```rust
//! use schema::load_all_schemas;
//!
//! let schema = load_all_schemas();
//! for cf in &schema.column_families {
//!     println!("Column Family: {}", cf.name);
//!     println!("  Category: {}", cf.category);
//!     println!("  Purpose: {}", cf.purpose);
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Complete schema definition for all Madara RocksDB column families.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    /// All column family schemas organized by category.
    pub column_families: Vec<ColumnFamilySchema>,
}

/// Schema definition for a single RocksDB column family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnFamilySchema {
    /// The RocksDB column family name (e.g., "block_info", "contract_storage").
    pub name: String,

    /// Category grouping for the column family.
    /// One of: "blocks", "transactions", "contracts", "classes", "tries", "meta", "messaging", "mempool", "events"
    pub category: String,

    /// Human-readable description of what this column family stores and its purpose.
    pub purpose: String,

    /// Schema definition for the key format.
    pub key: KeySchema,

    /// Schema definition for the value format.
    pub value: ValueSchema,

    /// Relationships to other column families.
    pub relationships: Vec<Relationship>,
}

/// Schema definition for a column family key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySchema {
    /// The Rust type used to represent the key (e.g., "u64", "Felt", "[u8; 32]").
    pub rust_type: String,

    /// Encoding format used for the key.
    /// One of: "big-endian", "little-endian", "raw", "bincode", "composite"
    pub encoding: String,

    /// Size of the key in bytes, if fixed-size. None for variable-size keys.
    pub size_bytes: Option<usize>,

    /// Detailed description of how the key is constructed.
    pub description: String,

    /// Example of the key in its raw byte representation (hex string).
    pub example_raw: String,

    /// Example of the key in its decoded/human-readable form.
    pub example_decoded: String,
}

/// Schema definition for a column family value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSchema {
    /// The Rust type used to represent the value.
    pub rust_type: String,

    /// Encoding format used for the value.
    /// One of: "bincode", "raw", "json"
    pub encoding: String,

    /// Detailed description of what the value contains.
    pub description: String,

    /// Fields within the value (for struct types).
    pub fields: Vec<FieldSchema>,
}

/// Schema definition for a field within a value struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Name of the field.
    pub name: String,

    /// Rust type of the field.
    pub rust_type: String,

    /// Description of what this field contains.
    pub description: String,
}

/// Relationship between column families.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Name of the target column family.
    pub target_cf: String,

    /// Type of relationship.
    /// One of: "inverse", "references", "contains", "indexed_by"
    pub relationship_type: String,

    /// Description of the relationship.
    pub description: String,
}

// Embedded YAML definitions
const BLOCKS_YAML: &str = include_str!("definitions/blocks.yaml");
const TRANSACTIONS_YAML: &str = include_str!("definitions/transactions.yaml");
const CONTRACTS_YAML: &str = include_str!("definitions/contracts.yaml");
const CLASSES_YAML: &str = include_str!("definitions/classes.yaml");
const BONSAI_YAML: &str = include_str!("definitions/bonsai.yaml");
const META_YAML: &str = include_str!("definitions/meta.yaml");
const MESSAGING_YAML: &str = include_str!("definitions/messaging.yaml");
const MEMPOOL_YAML: &str = include_str!("definitions/mempool.yaml");
const EVENTS_YAML: &str = include_str!("definitions/events.yaml");

/// Load all column family schemas from embedded YAML definitions.
///
/// This function parses all embedded YAML schema files and combines them
/// into a single `SchemaDefinition` containing all column families.
///
/// # Panics
///
/// Panics if any embedded YAML file fails to parse. This should never happen
/// in production as the YAML files are validated at compile time.
pub fn load_all_schemas() -> SchemaDefinition {
    let mut column_families = Vec::new();

    // Parse each YAML file and extend the column families vector
    let blocks: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(BLOCKS_YAML).expect("Failed to parse blocks.yaml");
    column_families.extend(blocks);

    let transactions: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(TRANSACTIONS_YAML).expect("Failed to parse transactions.yaml");
    column_families.extend(transactions);

    let contracts: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(CONTRACTS_YAML).expect("Failed to parse contracts.yaml");
    column_families.extend(contracts);

    let classes: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(CLASSES_YAML).expect("Failed to parse classes.yaml");
    column_families.extend(classes);

    let bonsai: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(BONSAI_YAML).expect("Failed to parse bonsai.yaml");
    column_families.extend(bonsai);

    let meta: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(META_YAML).expect("Failed to parse meta.yaml");
    column_families.extend(meta);

    let messaging: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(MESSAGING_YAML).expect("Failed to parse messaging.yaml");
    column_families.extend(messaging);

    let mempool: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(MEMPOOL_YAML).expect("Failed to parse mempool.yaml");
    column_families.extend(mempool);

    let events: Vec<ColumnFamilySchema> =
        serde_yaml::from_str(EVENTS_YAML).expect("Failed to parse events.yaml");
    column_families.extend(events);

    SchemaDefinition { column_families }
}

/// Load schemas for a specific category.
///
/// # Arguments
///
/// * `category` - The category to filter by (e.g., "blocks", "contracts", "tries")
///
/// # Returns
///
/// A `SchemaDefinition` containing only column families matching the specified category.
pub fn load_schemas_by_category(category: &str) -> SchemaDefinition {
    let all = load_all_schemas();
    let filtered: Vec<ColumnFamilySchema> = all
        .column_families
        .into_iter()
        .filter(|cf| cf.category == category)
        .collect();

    SchemaDefinition {
        column_families: filtered,
    }
}

/// Get a schema for a specific column family by name.
///
/// # Arguments
///
/// * `name` - The RocksDB column family name
///
/// # Returns
///
/// The schema for the specified column family, or None if not found.
pub fn get_schema_by_name(name: &str) -> Option<ColumnFamilySchema> {
    load_all_schemas()
        .column_families
        .into_iter()
        .find(|cf| cf.name == name)
}

/// Export all schemas to JSON format.
///
/// Useful for generating documentation or feeding into other tools.
pub fn export_to_json() -> String {
    let schema = load_all_schemas();
    serde_json::to_string_pretty(&schema).expect("Failed to serialize to JSON")
}

/// Export all schemas to YAML format.
///
/// Useful for generating documentation or configuration files.
pub fn export_to_yaml() -> String {
    let schema = load_all_schemas();
    serde_yaml::to_string(&schema).expect("Failed to serialize to YAML")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_schemas() {
        let schema = load_all_schemas();
        assert!(!schema.column_families.is_empty());

        // Verify we have all expected categories
        let categories: std::collections::HashSet<_> = schema
            .column_families
            .iter()
            .map(|cf| cf.category.as_str())
            .collect();

        assert!(categories.contains("blocks"));
        assert!(categories.contains("contracts"));
        assert!(categories.contains("classes"));
        assert!(categories.contains("tries"));
        assert!(categories.contains("meta"));
    }

    #[test]
    fn test_load_by_category() {
        let blocks = load_schemas_by_category("blocks");
        assert!(!blocks.column_families.is_empty());
        for cf in &blocks.column_families {
            assert_eq!(cf.category, "blocks");
        }
    }

    #[test]
    fn test_get_by_name() {
        let schema = get_schema_by_name("block_info");
        assert!(schema.is_some());
        let schema = schema.unwrap();
        assert_eq!(schema.name, "block_info");
    }

    #[test]
    fn test_export_json() {
        let json = export_to_json();
        assert!(!json.is_empty());
        // Verify it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_export_yaml() {
        let yaml = export_to_yaml();
        assert!(!yaml.is_empty());
        // Verify it's valid YAML
        let _: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
    }
}
