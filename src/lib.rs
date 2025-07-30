use std::error::Error;
use bincode::{Encode, Decode, encode_to_vec, decode_from_slice, config};
use redb::{Database, TableDefinition};

const TABLE_DEFINITION: TableDefinition<&[u8], &[u8]> = TableDefinition::new("kv");

type BoxedError = Box<dyn Error + Send + Sync>;

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
    use tempfile::tempdir;
    use std::sync::Arc;
    use std::thread;
    use bincode::{Decode, Encode};

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
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
        let value = TestValue { name: "alpha".into(), data: vec![1, 2, 3] };

        // Create
        db.insert(&key, &value).unwrap();

        // Read
        let fetched = db.get::<_, TestValue>(&key).unwrap().unwrap();
        assert_eq!(fetched, value);

        // Update
        let new_value = TestValue { name: "beta".into(), data: vec![4, 5, 6] };
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
    fn concurrent_reads_and_writes() {
        let dir = tempdir().unwrap();
        let db = Arc::new(QuadDB::new(dir.path().join("db.redb").to_str().unwrap()).unwrap());

        let num_threads = 10;
        let ops_per_thread = 20;
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let db = Arc::clone(&db);
            handles.push(thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = TestKey { id: thread_id * ops_per_thread + i };
                    let val = TestValue {
                        name: format!("worker-{thread_id}"),
                        data: vec![i as u8, thread_id as u8],
                    };

                    db.insert(&key, &val).unwrap();
                    let fetched = db.get::<_, TestValue>(&key).unwrap().unwrap();
                    assert_eq!(fetched, val);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}
