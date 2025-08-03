# QuaDB

[![Crates.io](https://img.shields.io/crates/v/quadb.svg?style=for-the-badge&logo=crates.io)](https://crates.io/crates/quadb)
[![Docs.rs](https://img.shields.io/badge/docs.rs-quadb-blue?style=for-the-badge&logo=docs.rs)](https://docs.rs/quadb)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-7F0000?style=for-the-badge&labelColor=000000&logoColor=white)](LICENSE)
[![Support on Ko-fi](https://img.shields.io/badge/ko--fi-Donate-999999?style=for-the-badge&logo=ko-fi&labelColor=333333)](https://ko-fi.com/stacksyndicate)

This project is in the very early stages of development and is nowhere near ready for use.

## Description

QuadDB is an asynchronous octree manager implemented over [`redb`](https://docs.rs/redb), designed for efficient storage and retrieval of spatial data from disk. Binary encoding is handled using [`bincode`](https://docs.rs/bincode) for compactness and speed.

## Project Vision

The goal of QuadDB is to provide a robust and efficient solution for managing and updating large spatial datasets, particularly in simulations. The use-case envisioned is for graduate students working on simulations that need to manipulate huge datasets involving particle systems or fluid dynamics where memory size is a constraint and access to a supercomputer is not feasible.

| Feature                       | Status                     |
|-------------------------------|----------------------------|
| $2^n$-Tree Backend            | :construction: In progress |
| Basic CRUD Operations         | :construction: In progress |
| Leaf Streaming                | :turtle: Planned           |
| Documentation                 | :turtle: Planned           |
| $kd$-Tree Backend             | :turtle: Planned           |

**$2^n$-Tree Backend**: An implementation of a generalised octree structure.

**Basic *CRUD* operations**: Creating, reading, updating and deleting entities in the database file.

**Leaf Streaming**: Dynamically retrieving leaf nodes from the database as requests are made through the CRUD interface.

**Documentation**: Robust documentation once the API is frozen.

**$kd$-Tree Backend**: An implementation of tree structures that do not form new leaf nodes by equally bipartitioning the space. Allows for custom bipartitioning schemes.

## License

This work is distributed under the MIT License. Dependencies either direct or indirect may have different licenses, so all of them have been reproduced in the THIRD-PARTY-LICENSES.md file at the root of this repository.
