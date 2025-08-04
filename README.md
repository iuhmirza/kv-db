# kv-db 🦀

An in-memory, multi-threaded key-value database server written in Rust. This repository demonstrates two common concurrency patterns for handling shared state: the **Actor Model** (using channels) and **Shared Memory Concurrency** (using `RwLock`).

The server listens on `127.0.0.1:6379` and communicates over a basic TCP protocol.

-----

## ✨ Features

  * **In-Memory Storage**: Data is stored in a `HashMap` and is not persisted.
  * **Multi-threaded**: Each client connection is handled in a separate thread.
  * **Basic Operations**: Supports `GET`, `PUT`, and `DELETE` operations.
  * **Concurrency Demonstrations**: Provides two distinct branches/files to showcase different concurrency models in Rust.
  * **Interactive CLI Client**: A simple client to interact with the server from the command line.

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

## 🚀 Usage

### Prerequisites

You must have the Rust toolchain installed. You can install it from [rust-lang.org](https://www.rust-lang.org/).

### Running the Server

1.  Clone the repository:

    ```bash
    git clone https://github.com/iuhmirza/kv-db
    cd kv-db
    ```

2.  Choose which server version you want to run. Assuming the server binaries are named `mp` and `lock`, you can run them as follows:

    **To run the Actor Model version:**

    ```bash
    cargo run --bin mp
    ```

    **To run the Shared Memory version:**

    ```bash
    cargo run --bin lock
    ```

    The server will start and listen on `127.0.0.1:6379`.

### 💻 Running the Client

The project includes an interactive command-line client to communicate with the server.

1.  First, ensure one of the server versions is running in a separate terminal.

2.  In a new terminal, run the client (assuming the client binary is named `client`):

    ```bash
    cargo run --bin client
    ```

3.  You can now send commands to the server. The available commands are:

      * `put <key> <value>`: Stores a value for the given key.
      * `get <key>`: Retrieves the value for the given key.
      * `del <key>`: Deletes the key and its value.
      * `exit`: Closes the client.

    **Example Session:**

    ```
    > put framework Actix
    > get framework
    framework:Actix
    > del framework
    > get framework
    framework:
    > exit
    ```

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