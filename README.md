# netperf_rust

A Rust implementation of the netperf network performance testing tool, aiming for parity with netperf 2.6.0.

## Overview

`netperf_rust` consists of two main components:
- `netperf_rust`: The client tool used to initiate network performance tests.
- `netserver_rust`: The server tool that listens for connections and handles test requests.

## Installation

To build the project, ensure you have Rust and Cargo installed, then run:

```bash
cargo build --release
```

The binaries will be available in `target/release/`.

## Usage

### netserver_rust

Start the server to listen for incoming test requests.

```bash
./target/release/netserver_rust [options]
```

**Options:**
- `-p <port>`: Port to listen on (default: 12865).
- `-V`: Print version information.
- `-h, --help`: Print help message.

### netperf_rust

Run tests against a `netserver_rust` instance.

```bash
./target/release/netperf_rust [global options] -- [test-specific options]
```

**Global Options:**
- `-H <host>`: Remote host to connect to (default: localhost).
- `-l <duration>`: Test duration in seconds (default: 10).
- `-t <type>`: Test type (default: TCP_STREAM).
- `-p <port>`: Port for the control connection (default: 12865).
- `-4`: Use IPv4.
- `-6`: Use IPv6.
- `-V`: Print version information.
- `-h, --help`: Print help message.

**Test-Specific Options (after `--`):**
- `-m <size>`: Message size for stream tests (default: 32768).
- `-M <size>`: Remote receive buffer size for stream tests.
- `-r <req,res>`: Request and response sizes for RR tests (default: 1,1).
- `-s <send,recv>`: Local send and receive socket buffer sizes.
- `-S <send,recv>`: Remote send and receive socket buffer sizes.
- `-D`: Set TCP_NODELAY (default: false).

**Supported formats:**
- **Sizespec**: `val` sets both, `val,` sets first, `,val` sets second.
- **Units**: Suffixes `K, M, G` (2^10, 2^20, 2^30 bytes) and `k, m, g` (10^3, 10^6, 10^9 bits).

## Supported Test Types

- `TCP_STREAM`: Measure TCP stream throughput.
- `TCP_RR`: Measure TCP request/response performance (transactions per second).
- `UDP_STREAM`: Measure UDP stream throughput.
- `UDP_RR`: Measure UDP request/response performance.
- `TCP_MAERTS`: Measure TCP stream throughput (server to client).
- `TCP_CRR`: Measure TCP connect/request/response performance.
- `TCP_CC`: Measure TCP connect/close performance.

## Examples

### Run a 10-second TCP stream test with 64KB message size:

1. Start the server:
   ```bash
   ./target/release/netserver_rust
   ```

2. Run the client:
   ```bash
   ./target/release/netperf_rust -H localhost -t TCP_STREAM -- -m 64K
   ```

### Run a TCP Request/Response test with 1KB request and 1KB response:

```bash
./target/release/netperf_rust -H localhost -t TCP_RR -- -r 1K,1K
```

## Cross Compilation

To cross-compile `netperf_rust` for other platforms (e.g., from macOS to Linux), the easiest way is to use [`cross`](https://github.com/cross-rs/cross).

### Using `cross`

1. Install `cross`:
   ```bash
   cargo install cross --git https://github.com/cross-rs/cross
   ```

2. Build for a specific target (e.g., x86_64 Linux):
   ```bash
   cross build --target x86_64-unknown-linux-gnu --release
   ```

### Using standard `cargo` (Manual)

If you prefer not to use `cross`, you can add the target via `rustup` and provide a linker.

1. Add the target:
   ```bash
   rustup target add x86_64-unknown-linux-gnu
   ```

2. Build with a cross-linker (requires a linker for the target to be installed on your host):
   ```bash
   cargo build --target x86_64-unknown-linux-gnu --release
   ```

Common targets:
- `x86_64-unknown-linux-gnu` (64-bit Linux)
- `aarch64-unknown-linux-gnu` (64-bit ARM Linux)
- `x86_64-pc-windows-msvc` (64-bit Windows)

## License

This project is licensed under the terms of the license found in the `netperf` directory (based on the original netperf license).
