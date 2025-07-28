# HTTP Server Improvement & Rewrite Checklist

## Phase 1: Improve Rust Server (Current Codebase)

### ✅ Basic Server Skeleton

- [x] TCP listener on `127.0.0.1:4221`
- [x] Accepts GET and POST
- [x] Parses headers
- [x] Responds with static/dynamic data
- [x] Handles file read/write

### ⬜ Connection Handling

- [ ] Support **Keep-Alive** (HTTP/1.1 persistent connections)
- [ ] Loop per connection to handle multiple requests
- [ ] Gracefully close connection on `Connection: close`

### ⬜ Reading Robustness

- [ ] Accumulate input until **full headers** are received (`\r\n\r\n`)
- [ ] Parse **chunked encoding** (later)
- [ ] Handle **partial body reads** if Content-Length > buffer

### ⬜ Writing Robustness

- [ ] Handle **partial writes** (track offset)
- [ ] Flush content correctly with large response bodies
- [ ] Add **timeout** for slow clients (optional)

### ⬜ Encoding & Compression

- [x] Gzip compression (already present)
- [ ] Support for `identity`, `br`, etc. (optional)

### ⬜ Errors and Logging

- [ ] Better error responses (e.g., 405 for unsupported method)
- [ ] Structured logging of requests and errors

---

## Phase 2: Architecture Upgrades in Rust

### ⬜ Thread Pool Model

- [ ] Fixed-size pool of worker threads
- [ ] Send incoming connections into a work queue
- [ ] Each thread handles connection lifecycle

### ⬜ Event-Driven Model

- [ ] Use `mio` or `tokio` for non-blocking I/O
- [ ] Register sockets with event loop
- [ ] Maintain per-connection state (headers/body buffer, response queue)
- [ ] Integrate timers for timeouts

### ⬜ TLS/HTTPS Support

- [ ] Add OpenSSL or Rustls
- [ ] Accept both HTTP and HTTPS (via separate listeners or ALPN)

---

## Phase 3: Rewrite from Scratch in C

### ⬜ Socket & Server Setup

- [ ] Use `socket()`, `bind()`, `listen()`, `accept()`
- [ ] Parse `argv[]` for options like `--directory`

### ⬜ Parsing & Protocol Handling

- [ ] Implement HTTP/1.0 and 1.1 request parsing manually
- [ ] Manually buffer input, detect `\r\n\r\n`
- [ ] Handle Content-Length and chunked transfer

### ⬜ Threading/Event Model

Choose one (or both for learning):

- [ ] **Thread Pool** with `pthread`, `mutex`, `queue`, `condition_variable`
- [ ] **Event Loop** using `epoll` (Linux), `select` (portable), or `kqueue` (BSD/macOS)

### ⬜ State Machines per Connection

- [ ] Track socket state: Reading, Writing, Idle, Closed
- [ ] Allocate buffers per socket (dynamic or fixed)

### ⬜ TLS

- [ ] Integrate **OpenSSL**
- [ ] Replace `read`/`write` with `SSL_read`/`SSL_write`
- [ ] Manage `SSL_CTX`, certificates, and keys

### ⬜ Cleanup and Error Handling

- [ ] Gracefully close sockets and free buffers
- [ ] Handle `SIGPIPE`, `SIGCHLD`, `SIGINT`

---

## Optional Advanced Features

- [ ] HTTP pipelining
- [ ] HTTP/2 via nghttp2 or hyper-h2 (for Rust)
- [ ] Virtual hosting
- [ ] Configurable routing
- [ ] Rate limiting and DoS protection
- [ ] Logging with timestamps
- [ ] File upload/multipart support
