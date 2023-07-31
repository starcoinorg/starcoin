use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("key {0} not found in store")]
    KeyNotFound(String),

    #[error("key {0} already exists in store")]
    KeyAlreadyExists(String),

    #[error("column family {0} not exist in db")]
    CFNotExist(String),

    #[error("IO error {0}")]
    DBIoError(String),

    #[error("rocksdb error {0}")]
    DbError(#[from] rocksdb::Error),

    #[error("encode error {0}")]
    EncodeError(String),

    #[error("decode error {0}")]
    DecodeError(String),

    #[error("ghostdag {0} duplicate blocks")]
    DAGDupBlocksError(String),
}

pub type StoreResult<T> = std::result::Result<T, StoreError>;

pub trait StoreResultExtensions<T> {
    fn unwrap_option(self) -> Option<T>;
}

impl<T> StoreResultExtensions<T> for StoreResult<T> {
    fn unwrap_option(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(StoreError::KeyNotFound(_)) => None,
            Err(err) => panic!("Unexpected store error: {err:?}"),
        }
    }
}

pub trait StoreResultEmptyTuple {
    fn unwrap_and_ignore_key_already_exists(self);
}

impl StoreResultEmptyTuple for StoreResult<()> {
    fn unwrap_and_ignore_key_already_exists(self) {
        match self {
            Ok(_) => (),
            Err(StoreError::KeyAlreadyExists(_)) => (),
            Err(err) => panic!("Unexpected store error: {err:?}"),
        }
    }
}
