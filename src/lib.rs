mod spacetree;
mod utils;

use bincode::{Decode, Encode, config, decode_from_slice, encode_to_vec};
use redb::{Database, TableDefinition};
use utils::BoxedError;

const TABLE_DEFINITION: TableDefinition<&[u8], &[u8]> = TableDefinition::new("kv");

struct QuadDB {
    db: Database,
}
impl QuadDB {
    pub fn new(path: &str) -> Result<Self, BoxedError> {
        let db = Database::create(path)?;
        let txn = db.begin_write()?;
        {
            let mut table = txn.open_table(TABLE_DEFINITION)?;
            table.insert("DUMMY".as_bytes(), "DUMMY".as_bytes())?;
        }
        txn.commit()?;
        Ok(QuadDB { db })
    }
    pub fn insert<K: Encode, V: Encode>(&self, key: &K, value: &V) -> Result<(), BoxedError> {
        let key_bytes = encode_to_vec(key, config::standard())?;
        let val_bytes = encode_to_vec(value, config::standard())?;
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(TABLE_DEFINITION)?;
            table.insert(key_bytes.as_slice(), val_bytes.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get<K: Encode, V: Decode<()>>(&self, key: K) -> Result<Option<V>, BoxedError> {
        let key_bytes = encode_to_vec(&key, config::standard())?;
        let txn = self.db.begin_read()?;
        let table = txn.open_table(TABLE_DEFINITION)?;
        if let Some(val_bytes) = table.get(&*key_bytes)? {
            let (decoded, _): (V, _) = decode_from_slice(val_bytes.value(), config::standard())?;
            Ok(Some(decoded))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{Decode, Encode};
    use tempfile::tempdir;

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
    struct TestKey {
        id: u32,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    struct TestValue {
        name: String,
        data: Vec<u8>,
    }

    #[test]
    fn basic_crud_operations() {
        let dir = tempdir().unwrap();
        let db = QuadDB::new(dir.path().join("db.redb").to_str().unwrap()).unwrap();

        let key = TestKey { id: 1 };
        let value = TestValue {
            name: "alpha".into(),
            data: vec![1, 2, 3],
        };

        // Create
        db.insert(&key, &value).unwrap();

        // Read
        let fetched = db.get::<_, TestValue>(&key).unwrap().unwrap();
        assert_eq!(fetched, value);

        // Update
        let new_value = TestValue {
            name: "beta".into(),
            data: vec![4, 5, 6],
        };
        db.insert(&key, &new_value).unwrap();
        let updated = db.get::<_, TestValue>(&key).unwrap().unwrap();
        assert_eq!(updated, new_value);
    }

    #[test]
    fn missing_key_returns_none() {
        let dir = tempdir().unwrap();
        let db = QuadDB::new(dir.path().join("db.redb").to_str().unwrap()).unwrap();

        let key = TestKey { id: 999 };
        let result = db.get::<_, TestValue>(&key).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn concurrent_reads_and_writes_with_contention() {
        use rand::{Rng, thread_rng};
        use std::{sync::Arc, thread, time::Duration};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let db = Arc::new(QuadDB::new(dir.path().join("db.redb").to_str().unwrap()).unwrap());

        let num_threads = 20;
        let ops_per_thread = 50;
        let shared_key = TestKey { id: 9999 };
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let db = Arc::clone(&db);
            handles.push(thread::spawn(move || {
                let mut rng = thread_rng();
                for i in 0..ops_per_thread {
                    // Thread-unique key
                    let key = TestKey {
                        id: thread_id * ops_per_thread + i,
                    };
                    let val = TestValue {
                        name: format!("worker-{thread_id}"),
                        data: vec![i as u8, thread_id as u8],
                    };

                    db.insert(&key, &val).expect("insert failed (unique key)");
                    let fetched = db
                        .get::<_, TestValue>(&key)
                        .expect("get failed (unique key)")
                        .expect("missing value");
                    assert_eq!(fetched, val, "unique key mismatch");

                    // Shared key (last write wins)
                    let shared_val = TestValue {
                        name: format!("shared-{thread_id}-{i}"),
                        data: vec![i as u8],
                    };

                    db.insert(&shared_key, &shared_val)
                        .expect("insert failed (shared key)");
                    let fetched_shared = db
                        .get::<_, TestValue>(&shared_key)
                        .expect("get failed (shared key)")
                        .expect("shared key missing");

                    assert!(
                        fetched_shared.name.starts_with("shared-"),
                        "shared key corrupt: {:?}",
                        fetched_shared
                    );

                    thread::sleep(Duration::from_millis(rng.gen_range(0..5)));
                }

                Ok::<(), String>(())
            }));
        }

        for handle in handles {
            handle
                .join()
                .expect("thread panicked")
                .expect("thread error");
        }

        // Final check: shared key should still be valid
        let final_val = db.get::<_, TestValue>(&shared_key.clone()).unwrap().unwrap();
        assert!(
            final_val.name.starts_with("shared-"),
            "final shared value corrupt"
        );
    }
}
