# QuadDB

QuadDB is a minimal typed key-value octree store built on top of [`redb`](https://docs.rs/redb), using efficient binary encoding via [`bincode`](https://docs.rs/bincode).

## Project Vision

QuadDB is the foundation for a generalised octree-based simulation engine. The long-term goal is to support **arbitrary-dimensional spatial indexing**, enabling **real-time simulations** that **stream data to and from disk** efficiently—without requiring everything to fit in memory.

## Features & Tradeoffs

- Typed keys and values implementing `bincode::Encode` / `Decode`
- Simple insert/get interface
- Concurrent read/write safe
- Built on the high-performance `redb` embedded database
- Uses `dashmap` for in-memory caching
- Trades in-memory performance for scalability
