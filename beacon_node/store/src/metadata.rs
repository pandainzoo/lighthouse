use crate::{DBColumn, Error, StoreItem};
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use types::{Checkpoint, Hash256, Slot};

pub const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion(21);

// All the keys that get stored under the `BeaconMeta` column.
//
// We use `repeat_byte` because it's a const fn.
pub const SCHEMA_VERSION_KEY: Hash256 = Hash256::repeat_byte(0);
pub const CONFIG_KEY: Hash256 = Hash256::repeat_byte(1);
pub const SPLIT_KEY: Hash256 = Hash256::repeat_byte(2);
pub const PRUNING_CHECKPOINT_KEY: Hash256 = Hash256::repeat_byte(3);
pub const COMPACTION_TIMESTAMP_KEY: Hash256 = Hash256::repeat_byte(4);
pub const ANCHOR_INFO_KEY: Hash256 = Hash256::repeat_byte(5);
pub const BLOB_INFO_KEY: Hash256 = Hash256::repeat_byte(6);

/// State upper limit value used to indicate that a node is not storing historic states.
pub const STATE_UPPER_LIMIT_NO_RETAIN: Slot = Slot::new(u64::MAX);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaVersion(pub u64);

impl SchemaVersion {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl StoreItem for SchemaVersion {
    fn db_column() -> DBColumn {
        DBColumn::BeaconMeta
    }

    fn as_store_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }

    fn from_store_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(SchemaVersion(u64::from_ssz_bytes(bytes)?))
    }
}

/// The checkpoint used for pruning the database.
///
/// Updated whenever pruning is successful.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PruningCheckpoint {
    pub checkpoint: Checkpoint,
}

impl StoreItem for PruningCheckpoint {
    fn db_column() -> DBColumn {
        DBColumn::BeaconMeta
    }

    fn as_store_bytes(&self) -> Vec<u8> {
        self.checkpoint.as_ssz_bytes()
    }

    fn from_store_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(PruningCheckpoint {
            checkpoint: Checkpoint::from_ssz_bytes(bytes)?,
        })
    }
}

/// The last time the database was compacted.
pub struct CompactionTimestamp(pub u64);

impl StoreItem for CompactionTimestamp {
    fn db_column() -> DBColumn {
        DBColumn::BeaconMeta
    }

    fn as_store_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }

    fn from_store_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(CompactionTimestamp(u64::from_ssz_bytes(bytes)?))
    }
}

/// Database parameters relevant to weak subjectivity sync.
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct AnchorInfo {
    /// The slot at which the anchor state is present and which we cannot revert.
    pub anchor_slot: Slot,
    /// The slot from which historical blocks are available (>=).
    pub oldest_block_slot: Slot,
    /// The block root of the next block that needs to be added to fill in the history.
    ///
    /// Zero if we know all blocks back to genesis.
    pub oldest_block_parent: Hash256,
    /// The slot from which historical states are available (>=).
    pub state_upper_limit: Slot,
    /// The slot before which historical states are available (<=).
    pub state_lower_limit: Slot,
}

impl AnchorInfo {
    /// Returns true if the block backfill has completed.
    /// This is a comparison between the oldest block slot and the target backfill slot (which is
    /// likely to be the closest WSP).
    pub fn block_backfill_complete(&self, target_slot: Slot) -> bool {
        self.oldest_block_slot <= target_slot
    }
}

impl StoreItem for AnchorInfo {
    fn db_column() -> DBColumn {
        DBColumn::BeaconMeta
    }

    fn as_store_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }

    fn from_store_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self::from_ssz_bytes(bytes)?)
    }
}

/// Database parameters relevant to blob sync.
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, Serialize, Deserialize, Default)]
pub struct BlobInfo {
    /// The slot after which blobs are or *will be* available (>=).
    ///
    /// If this slot is in the future, then it is the first slot of the Deneb fork, from which blobs
    /// will be available.
    ///
    /// If the `oldest_blob_slot` is `None` then this means that the Deneb fork epoch is not yet
    /// known.
    pub oldest_blob_slot: Option<Slot>,
    /// A separate blobs database is in use (deprecated, always `true`).
    pub blobs_db: bool,
}

impl StoreItem for BlobInfo {
    fn db_column() -> DBColumn {
        DBColumn::BeaconMeta
    }

    fn as_store_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }

    fn from_store_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self::from_ssz_bytes(bytes)?)
    }
}
