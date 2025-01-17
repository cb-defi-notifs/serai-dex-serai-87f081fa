use std::collections::HashMap;

use zeroize::Zeroizing;
use rand::{RngCore, rngs::OsRng};

use ciphersuite::{group::ff::Field, Ciphersuite, Ristretto};

use serai_db::MemDb;

use crate::{
  ACCOUNT_MEMPOOL_LIMIT, Transaction, Mempool,
  tests::{SignedTransaction, signed_transaction},
};

fn new_mempool<T: Transaction>() -> ([u8; 32], MemDb, Mempool<MemDb, T>) {
  let mut genesis = [0; 32];
  OsRng.fill_bytes(&mut genesis);
  let db = MemDb::new();
  (genesis, db.clone(), Mempool::new(db, genesis))
}

#[test]
fn mempool_addition() {
  let (genesis, db, mut mempool) = new_mempool::<SignedTransaction>();

  let key = Zeroizing::new(<Ristretto as Ciphersuite>::F::random(&mut OsRng));

  let first_tx = signed_transaction(&mut OsRng, genesis, &key, 0);
  let signer = first_tx.1.signer;
  assert_eq!(mempool.next_nonce(&signer), None);

  // Add TX 0
  let mut blockchain_next_nonces = HashMap::from([(signer, 0)]);
  assert!(mempool.add(&blockchain_next_nonces, true, first_tx.clone()));
  assert_eq!(mempool.next_nonce(&signer), Some(1));

  // Test reloading works
  assert_eq!(mempool, Mempool::new(db, genesis));

  // Adding it again should fail
  assert!(!mempool.add(&blockchain_next_nonces, true, first_tx.clone()));

  // Do the same with the next nonce
  let second_tx = signed_transaction(&mut OsRng, genesis, &key, 1);
  assert!(mempool.add(&blockchain_next_nonces, true, second_tx.clone()));
  assert_eq!(mempool.next_nonce(&signer), Some(2));
  assert!(!mempool.add(&blockchain_next_nonces, true, second_tx.clone()));

  // If the mempool doesn't have a nonce for an account, it should successfully use the
  // blockchain's
  let second_key = Zeroizing::new(<Ristretto as Ciphersuite>::F::random(&mut OsRng));
  let tx = signed_transaction(&mut OsRng, genesis, &second_key, 2);
  let second_signer = tx.1.signer;
  assert_eq!(mempool.next_nonce(&second_signer), None);
  blockchain_next_nonces.insert(second_signer, 2);
  assert!(mempool.add(&blockchain_next_nonces, true, tx.clone()));
  assert_eq!(mempool.next_nonce(&second_signer), Some(3));

  // Getting a block should work
  assert_eq!(mempool.block(&blockchain_next_nonces).len(), 3);

  // If the blockchain says an account had its nonce updated, it should cause a prune
  blockchain_next_nonces.insert(signer, 1);
  let mut block = mempool.block(&blockchain_next_nonces);
  assert_eq!(block.len(), 2);
  assert!(!block.iter().any(|tx| tx.hash() == first_tx.hash()));
  assert_eq!(mempool.txs(), &block.drain(..).map(|tx| (tx.hash(), tx)).collect::<HashMap<_, _>>());

  // Removing should also successfully prune
  mempool.remove(&tx.hash());
  assert_eq!(mempool.txs(), &HashMap::from([(second_tx.hash(), second_tx)]));
}

#[test]
fn too_many_mempool() {
  let (genesis, _, mut mempool) = new_mempool::<SignedTransaction>();

  let key = Zeroizing::new(<Ristretto as Ciphersuite>::F::random(&mut OsRng));
  let signer = signed_transaction(&mut OsRng, genesis, &key, 0).1.signer;

  // We should be able to add transactions up to the limit
  for i in 0 .. ACCOUNT_MEMPOOL_LIMIT {
    assert!(mempool.add(
      &HashMap::from([(signer, 0)]),
      false,
      signed_transaction(&mut OsRng, genesis, &key, i)
    ));
  }
  // Yet adding more should fail
  assert!(!mempool.add(
    &HashMap::from([(signer, 0)]),
    false,
    signed_transaction(&mut OsRng, genesis, &key, ACCOUNT_MEMPOOL_LIMIT)
  ));
}
