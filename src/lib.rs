use bincode::{Decode, Encode};
use paraxis::ZOctree;
use redb::{Database, TableDefinition, Value};

pub struct QuaDB<T, const D: usize>
where T: 'static + Value + Send + Sync + Clone + Copy + Encode + Decode<T> {
	table_def: TableDefinition<'static, u64, Vec<u8>>,
	db: Database,
	octree: ZOctree<T>
}
impl<T, const D: usize> QuaDB<T, D> 
where T: 'static + Value + Send + Sync + Copy + Clone + Encode + Decode<T> + Decode<()> {
	pub fn new(path: String) -> Self {
		let table_def = TableDefinition::new("data");
		let db = Database::create(path).expect("Could not create database.");
		Self { table_def, db, octree: ZOctree::new(D as u16)}
	}
	pub fn insert(&mut self, position: &[u16; D], value: T) {
		let write_txn = self.db.begin_write().expect("Write transaction creation failed.");
		{
			let mut table = write_txn.open_table(self.table_def).expect("Could not open table.");
			let bytes = bincode::encode_to_vec(value, bincode::config::standard()).expect("Failed to encode value into binary.");
			table.insert(self.octree.encode(position), bytes).expect("Failed to insert value into table.");
		}
		write_txn.commit().expect("Failed to commit insertion.");
	}
	pub fn remove(&mut self, position: &[u16; D]) {
		let write_txn = self.db.begin_write().expect("Write transaction creation failed.");
		{
			let mut table = write_txn.open_table(self.table_def).expect("Could not open table.");
			table.remove(self.octree.encode(position)).expect("Failed to remove value from table.");
		}
		write_txn.commit().expect("Failed to commit insertion.");
	}
	pub fn stream(&mut self, position: &[u16; D], radius: usize) {
		self.octree.clear();
		let min_coords: [u16; D] = {
			let mut a = [0u16; D];
			for i in 0..D { a[i] = position[i].saturating_sub(radius as u16); } a
		};
		let max_coords: [u16; D] = {
            let mut a = [0u16; D];
            for i in 0..D { a[i] = position[i].saturating_add(radius as u16); } a
        };
		let start = self.octree.encode(&min_coords);
        let end = self.octree.encode(&max_coords);

		let read_txn = self.db.begin_write().expect("Read transaction creation failed");
        let mut table = read_txn.open_table(self.table_def).expect("Could not open table.");

		let entries = table
			.extract_from_if(start..=end, |_,_| true)
			.expect("Table extraction failed.");
		for res in entries {
			let (key, val) = res.expect("Iterator yielded an error.");
			let (decoded, _): (T, _) = bincode::decode_from_slice(&val.value(), bincode::config::standard()).expect("Failed to decode.");
			let coords_vec = self.octree.decode(key.value());
			let coords_array: [u16; D] = coords_vec.try_into().expect("Dimension mismatch.");
			self.octree.insert(decoded, &coords_array);
		}
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Helper to create a QuaDB for testing using u32 as T and D=3
    fn make_db(path: String) -> QuaDB<u32, 3> {
        QuaDB::new(path)
    }

    #[test]
    fn insert_get_remove_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("testdb.redb").to_string_lossy().into_owned();
        let mut db = make_db(path);

        let pos: [u16; 3] = [10, 20, 30];
        let value: u32 = 42;

        // insert
        db.insert(&pos, value);
		db.stream(&pos, 1);

        // octree should contain it immediately
        let from_oct = db.octree.get(&pos);
        assert_eq!(from_oct, Some(value));

        // remove
        db.remove(&pos);
		db.stream(&pos, 1);

        // octree must not have it
        assert!(db.octree.get(&pos).is_none());

        // stream around pos should not re-populate (since DB entry removed)
        db.stream(&pos, 1);
        assert!(db.octree.get(&pos).is_none());
    }

    #[test]
    fn stream_populates_octree() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("streamdb.redb").to_string_lossy().into_owned();
        let mut db = make_db(path);

        // insert multiple points
        let points = vec![
            ([0u16, 0, 0], 1u32),
            ([1u16, 1, 1], 2u32),
            ([5u16, 5, 5], 3u32),
            ([10u16, 10, 10], 4u32),
        ];

        for (p, v) in &points {
            db.insert(p, *v);
        }

        // clear octree to emulate cold start
        // (depends on ZOctree API; here we recreate QuaDB.octree)
        db.octree = ZOctree::new(3);

        // stream region around [1,1,1] radius 1 -> should load points at [0,0,0], [1,1,1]
        db.stream(&[1u16, 1, 1], 1);

        assert_eq!(db.octree.get(&[0u16,0,0]), Some(1u32));
        assert_eq!(db.octree.get(&[1u16,1,1]), Some(2u32));
        assert!(db.octree.get(&[5u16,5,5]).is_none());

        // stream a larger region
        db.stream(&[5u16,5,5], 10);
        assert_eq!(db.octree.get(&[10u16,10,10]), Some(4u32));
    }

    #[test]
    fn load_on_miss_reads_db() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("loaddb.redb").to_string_lossy().into_owned();
        let mut db = make_db(path);

        let pos: [u16; 3] = [7, 8, 9];
        let value: u32 = 99;

        // insert to DB (and octree)
        db.insert(&pos, value);

        // recreate octree to simulate it missing data in memory
        db.octree = ZOctree::new(3);
        assert!(db.octree.get(&pos).is_none());

        // call stream with radius 0 (center only) or implement get_or_load and call that
        db.stream(&pos, 0);

        // now octree should contain it after stream
        assert_eq!(db.octree.get(&pos), Some(value));
    }
}
