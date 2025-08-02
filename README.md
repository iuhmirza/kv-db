# kv-db 🦀

An in-memory, multi-threaded key-value database server written in Rust. This repository demonstrates two common concurrency patterns for handling shared state: the **Actor Model** (using channels) and **Shared Memory Concurrency** (using `RwLock`).

The server listens on `127.0.0.1:6379` and communicates over a basic TCP protocol.

-----

## ✨ Features

  * **In-Memory Storage**: Data is stored in a `HashMap` and is not persisted.
  * **Multi-threaded**: Each client connection is handled in a separate thread.
  * **Basic Operations**: Supports `GET`, `PUT`, and `DELETE` operations.
  * **Concurrency Demonstrations**: Provides two distinct branches/files to showcase different concurrency models in Rust.

-----

## Concurrency Implementations

This repository contains two different implementations of the server, each showcasing a different approach to managing concurrent access to the key-value store.

### 1\. Actor Model (`mp.rs`)

This version uses a dedicated worker thread that "owns" the `HashMap`. All other threads are client handlers that communicate with the worker thread by sending commands through a multi-producer, single-consumer (`mpsc`) channel.

  * **How it works**: Client threads send `Command` enums (`Get`, `Put`, `Delete`) to the worker thread.
  * **Pros**: Avoids the need for explicit locks (`Mutex` or `RwLock`), preventing potential deadlocks. Access to the data is serialized through the channel's message queue.
  * **Cons**: The single worker thread can become a bottleneck if the command processing is slow or the number of client requests is extremely high.

### 2\. Shared Memory with `RwLock` (`lock.rs`)

This version wraps the `HashMap` in an `Arc<RwLock<...>>`. An `Arc` (Atomically Referenced Counter) allows multiple threads to share ownership of the data, and an `RwLock` (Read-Write Lock) allows for either multiple readers or one writer at a time.

  * **How it works**: Client threads directly acquire a lock on the `HashMap` to perform operations. A read lock is used for `GET`, and a write lock is used for `PUT` and `DELETE`.
  * **Pros**: Allows for concurrent reads, which can significantly improve performance in read-heavy workloads.
  * **Cons**: Can introduce lock contention. If writes are frequent, threads may spend time waiting for the write lock to be released.

-----

## 🚀 Getting Started

### Prerequisites

You must have the Rust toolchain installed. You can install it from [rust-lang.org](https://www.rust-lang.org/).

### Running the Server

1.  Clone the repository:

    ```bash
    git clone https://github.com/iuhmirza/kv-db
    cd kv-db
    ```

2.  Choose which version you want to run. You'll likely need to configure your `Cargo.toml` to specify the binaries if both `main.rs` files are in the `src/bin` directory. Assuming they are named `mp.rs` and `lock.rs`:

    **To run the Actor Model version:**

    ```bash
    cargo run --bin mp
    ```

    **To run the Shared Memory version:**

    ```bash
    cargo run --bin lock
    ```

    The server will start and listen on `127.0.0.1:6379`.

-----

## 🔌 Network Protocol

The server uses a simple custom binary protocol. Each message part is prefixed with a single byte indicating its length.

  * **PUT**: `+<key_len><key><value_len><value>`

      * `+`: Command identifier.
      * `<key_len>`: 1 byte, length of the key.
      * `<key>`: The key string.
      * `<value_len>`: 1 byte, length of the value.
      * `<value>`: The value string.

  * **GET**: `=<key_len><key>`

      * `=`: Command identifier.
      * `<key_len>`: 1 byte, length of the key.
      * `<key>`: The key string.

  * **DELETE**: `-<key_len><key>`

      * `-`: Command identifier.
      * `<key_len>`: 1 byte, length of the key.
      * `<key>`: The key string.

  * **Response**: `<value_len><value>`

      * For `GET`, returns the value.
      * For `PUT`, returns the old value.
      * For `DELETE`, returns the deleted value.
      * If a key is not found, returns an empty string with length 0.
-----