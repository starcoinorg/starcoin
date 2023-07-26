mod access;
mod cache;
mod consensus_ghostdag;
mod consensus_header;
mod consensus_reachability;
mod consensus_relations;
mod db;
mod errors;
mod item;
mod writer;

pub mod prelude {
    use crate::{db, errors};

    pub use super::{
        access::CachedDbAccess,
        cache::Cache,
        item::CachedDbItem,
        writer::{BatchDbWriter, DbWriter, DirectDbWriter},
    };
    pub use db::{FlexiDagStorage, FlexiDagStorageConfig};
    pub use errors::{StoreError, StoreResult, StoreResultEmptyTuple, StoreResultExtensions};
}

pub mod consensus {
    pub use super::{
        consensus_ghostdag::*, consensus_header::*, consensus_reachability::*,
        consensus_relations::*,
    };
}
