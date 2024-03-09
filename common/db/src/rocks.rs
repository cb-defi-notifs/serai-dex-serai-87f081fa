use std::sync::Arc;

use rocksdb::{
  DBCompressionType, ThreadMode, SingleThreaded, LogLevel, WriteOptions, Transaction, Options,
  TransactionDB,
};

use crate::*;

impl<T: ThreadMode> Get for Transaction<'_, TransactionDB<T>> {
  fn get(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
    self.get(key).expect("couldn't read from RocksDB via transaction")
  }
}
impl<T: ThreadMode> DbTxn for Transaction<'_, TransactionDB<T>> {
  fn put(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
    Transaction::put(self, key, value).expect("couldn't write to RocksDB via transaction")
  }
  fn del(&mut self, key: impl AsRef<[u8]>) {
    self.delete(key).expect("couldn't delete from RocksDB via transaction")
  }
  fn commit(self) {
    Transaction::commit(self).expect("couldn't commit to RocksDB via transaction")
  }
}

impl<T: ThreadMode> Get for Arc<TransactionDB<T>> {
  fn get(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
    TransactionDB::get(self, key).expect("couldn't read from RocksDB")
  }
}
impl<T: ThreadMode + 'static> Db for Arc<TransactionDB<T>> {
  type Transaction<'a> = Transaction<'a, TransactionDB<T>>;
  fn txn(&mut self) -> Self::Transaction<'_> {
    let mut opts = WriteOptions::default();
    opts.set_sync(true);
    self.transaction_opt(&opts, &Default::default())
  }
}

pub type RocksDB = Arc<TransactionDB<SingleThreaded>>;
pub fn new_rocksdb(path: &str) -> RocksDB {
  let mut options = Options::default();
  options.create_if_missing(true);
  options.set_compression_type(DBCompressionType::Zstd);

  options.set_wal_compression_type(DBCompressionType::Zstd);
  // 10 MB
  options.set_max_total_wal_size(10 * 1024 * 1024);
  options.set_wal_size_limit_mb(10);

  options.set_log_level(LogLevel::Warn);
  // 1 MB
  options.set_max_log_file_size(1024 * 1024);
  options.set_recycle_log_file_num(1);

  Arc::new(TransactionDB::open(&options, &Default::default(), path).unwrap())
}
