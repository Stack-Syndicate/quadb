pub mod spacetree;
mod utils;
pub mod prelude;

use bincode::{Decode, Encode, config, decode_from_slice, encode_to_vec};
use redb::{ReadableDatabase, TableDefinition, Database};
use utils::BoxedError;

use crate::spacetree::SpaceTree;

use crate::spacetree::Entity;

const TABLE_DEFINITION: TableDefinition<&[u8], &[u8]> = TableDefinition::new("kv");

pub struct QuadDB<const D: usize, E: Entity> {
    db: Database,
    st: SpaceTree<D, E>
}
impl<const D: usize, E: Entity> QuadDB<D, E> {
    pub fn new(path: &str) -> Result<Self, BoxedError> {
        let db = Database::create(path)?;
        let txn = db.begin_write()?;
        {
            let mut table = txn.open_table(TABLE_DEFINITION)?;
            table.insert("DUMMY".as_bytes(), "DUMMY".as_bytes())?;
        }
        txn.commit()?;
        let st = SpaceTree::<D, E>::new();
        Ok(QuadDB { db, st })
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

