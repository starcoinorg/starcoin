use crate::{db_storage::transaction_storage::DBTransactionStorage, storage::ColumnFamilyName};
use anyhow::bail;
use rocksdb::{OptimisticTransactionDB, SingleThreaded};
use starcoin_config::{temp_dir, RocksdbConfig};

fn init_transaction_db(
    columns: &[ColumnFamilyName],
) -> anyhow::Result<OptimisticTransactionDB<SingleThreaded>> {
    // db config
    let mut cf_opts = rocksdb::Options::default();
    cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
    cf_opts.create_if_missing(true);
    cf_opts.create_missing_column_families(true);

    let temp_path = temp_dir();
    let db: OptimisticTransactionDB<SingleThreaded> = OptimisticTransactionDB::open_cf_descriptors(
        &cf_opts,
        temp_path,
        columns.iter().map(|col| {
            let mut cf_opts = rocksdb::Options::default();
            cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
            rocksdb::ColumnFamilyDescriptor::new((*col).to_string(), cf_opts)
        }),
    )?;

    anyhow::Ok(db)
}

fn init_columns() -> anyhow::Result<Vec<ColumnFamilyName>> {
    const TEST_FOR_TRANSACTION1_DATA_CF: &str = "test-for-transaction1-data";
    const TEST_FOR_TRANSACTION2_DATA_CF: &str = "test-for-transaction2-data";
    const TEST_FOR_TRANSACTION3_DATA_CF: &str = "test-for-transaction3-data";

    let columns: Vec<ColumnFamilyName> = vec![
        TEST_FOR_TRANSACTION1_DATA_CF,
        TEST_FOR_TRANSACTION2_DATA_CF,
        TEST_FOR_TRANSACTION3_DATA_CF,
    ];
    anyhow::Ok(columns)
}

#[test]
fn test_transaction_write_in_single_mode() -> anyhow::Result<()> {
    let columns = init_columns()?;
    let db = init_transaction_db(&columns)?;

    let column1 = db.cf_handle(columns[0]).unwrap_or_else(|| {
        panic!(
            "check the column name: {0} that maybe not defined",
            columns[0]
        )
    });
    let column2 = db.cf_handle(columns[1]).unwrap_or_else(|| {
        panic!(
            "check the column name: {0} that maybe not defined",
            columns[1]
        )
    });
    let column3 = db.cf_handle(columns[2]).unwrap_or_else(|| {
        panic!(
            "check the column name: {0} that maybe not defined",
            columns[2]
        )
    });

    // write and commit
    let transaction = db.transaction();

    transaction.put_cf(column1, b"jack_key", b"jack")?;
    transaction.put_cf(column2, b"loves_key", b"loves")?;
    transaction.put_cf(column3, b"rose_key", b"rose")?;

    transaction.commit()?;

    if let Some(column_value) = db.get_cf(column1, b"jack_key")? {
        assert_eq!(column_value, b"jack");
    } else {
        bail!("failed to get the value from the column1");
    }

    if let Some(column_value) = db.get_cf(column2, b"loves_key")? {
        assert_eq!(column_value, b"loves");
    } else {
        bail!("failed to get the value from the column2");
    }

    if let Some(column_value) = db.get_cf(column3, b"rose_key")? {
        assert_eq!(column_value, b"rose");
    } else {
        bail!("failed to get the value from the column3");
    }

    // write and rollback
    let transaction = db.transaction();

    transaction.put_cf(column1, b"jack_key", b"jacky")?;
    transaction.put_cf(column2, b"loves_key", b"loves eternally")?;
    transaction.put_cf(column3, b"rose_key", b"claire")?;

    transaction.rollback()?;

    if let Some(column_value) = db.get_cf(column1, b"jack_key")? {
        assert_eq!(column_value, b"jack");
    } else {
        bail!("failed to get the value from the column1");
    }

    if let Some(column_value) = db.get_cf(column2, b"loves_key")? {
        assert_eq!(column_value, b"loves");
    } else {
        bail!("failed to get the value from the column2");
    }

    if let Some(column_value) = db.get_cf(column3, b"rose_key")? {
        assert_eq!(column_value, b"rose");
    } else {
        bail!("failed to get the value from the column3");
    }

    // write again and commit this time
    let transaction = db.transaction();

    transaction.put_cf(column1, b"jack_key", b"jacky")?;
    transaction.put_cf(column2, b"loves_key", b"loves eternally")?;
    transaction.put_cf(column3, b"rose_key", b"claire")?;

    transaction.commit()?;

    if let Some(column_value) = db.get_cf(column1, b"jack_key")? {
        assert_eq!(column_value, b"jacky");
    } else {
        bail!("failed to get the value from the column1");
    }

    if let Some(column_value) = db.get_cf(column2, b"loves_key")? {
        assert_eq!(column_value, b"loves eternally");
    } else {
        bail!("failed to get the value from the column2");
    }

    if let Some(column_value) = db.get_cf(column3, b"rose_key")? {
        assert_eq!(column_value, b"claire");
    } else {
        bail!("failed to get the value from the column3");
    }

    anyhow::Ok(())
}

#[test]
fn test_transaction_write_in_batch_mode() -> anyhow::Result<()> {
    let columns = init_columns()?;
    let db = init_transaction_db(&columns)?;

    let column1 = db.cf_handle(columns[0]).unwrap_or_else(|| {
        panic!(
            "check the column name: {0} that maybe not defined",
            columns[0]
        )
    });
    let column2 = db.cf_handle(columns[1]).unwrap_or_else(|| {
        panic!(
            "check the column name: {0} that maybe not defined",
            columns[1]
        )
    });
    let column3 = db.cf_handle(columns[2]).unwrap_or_else(|| {
        panic!(
            "check the column name: {0} that maybe not defined",
            columns[2]
        )
    });

    // write and commit
    let transaction = db.transaction();

    let mut batch = transaction.get_writebatch();

    batch.put_cf(column1, b"jack_key", b"jack");
    batch.put_cf(column2, b"loves_key", b"loves");
    batch.put_cf(column3, b"rose_key", b"rose");

    db.write(batch)?;

    if let Some(column_value) = db.get_cf(column1, b"jack_key")? {
        assert_eq!(column_value, b"jack");
    } else {
        bail!("failed to get the value from the column1");
    }

    if let Some(column_value) = db.get_cf(column2, b"loves_key")? {
        assert_eq!(column_value, b"loves");
    } else {
        bail!("failed to get the value from the column2");
    }

    if let Some(column_value) = db.get_cf(column3, b"rose_key")? {
        assert_eq!(column_value, b"rose");
    } else {
        bail!("failed to get the value from the column3");
    }

    // write and rollback
    let transaction = db.transaction();

    {
        let mut batch = transaction.get_writebatch();

        batch.put_cf(column1, b"jack_key", b"jacky");
        batch.put_cf(column2, b"loves_key", b"loves eternally");
        batch.put_cf(column3, b"rose_key", b"claire");
    }

    // no writing
    // ...

    if let Some(column_value) = db.get_cf(column1, b"jack_key")? {
        assert_eq!(column_value, b"jack");
    } else {
        bail!("failed to get the value from the column1");
    }

    if let Some(column_value) = db.get_cf(column2, b"loves_key")? {
        assert_eq!(column_value, b"loves");
    } else {
        bail!("failed to get the value from the column2");
    }

    if let Some(column_value) = db.get_cf(column3, b"rose_key")? {
        assert_eq!(column_value, b"rose");
    } else {
        bail!("failed to get the value from the column3");
    }

    // write again and commit this time
    let transaction = db.transaction();

    let mut batch = transaction.get_writebatch();

    batch.put_cf(column1, b"jack_key", b"jacky");
    batch.put_cf(column2, b"loves_key", b"loves eternally");
    batch.put_cf(column3, b"rose_key", b"claire");

    db.write(batch)?;

    if let Some(column_value) = db.get_cf(column1, b"jack_key")? {
        assert_eq!(column_value, b"jacky");
    } else {
        bail!("failed to get the value from the column1");
    }

    if let Some(column_value) = db.get_cf(column2, b"loves_key")? {
        assert_eq!(column_value, b"loves eternally");
    } else {
        bail!("failed to get the value from the column2");
    }

    if let Some(column_value) = db.get_cf(column3, b"rose_key")? {
        assert_eq!(column_value, b"claire");
    } else {
        bail!("failed to get the value from the column3");
    }

    anyhow::Ok(())
}

#[test]
fn test_db_transaction_storage() -> anyhow::Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let _storage = DBTransactionStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap();

    anyhow::Ok(())
}