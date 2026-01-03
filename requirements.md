# Netperf Rust Requirements

This document outlines the requirements for the `netperf_rust` project, aiming for parity with the original C/C++ implementation of Netperf 2.7.0.

## 1. Project Goals
- Provide a high-performance network benchmarking tool written in Rust.
- Achieve functional parity with Netperf 2.7.0 for core test types.
- Improve upon the original implementation by leveraging Rust's safety features.
- Address known security and robustness issues in the original and initial Rust implementation.

## 2. Command-Line Interface (CLI)

### 2.1 General Conventions
- **Global vs. Test-Specific Options**: Options are divided into global and test-specific categories, separated by `--`.
  `netperf_rust <global options> -- <test-specific options>`
- **Sizespec**: A comma-separated list `param1,param2`. 
  - `val` sets both to `val`.
  - `val,` sets only `param1`.
  - `,val` sets only `param2`.
- **Optionspec**: Similar to sizespec, but `val` sets only `param1`.
- **Units**: Support for suffixes:
  - `K, M, G`: 2^10, 2^20, 2^30 bytes.
  - `k, m, g`: 10^3, 10^6, 10^9 bits.

### 2.2 Global Options (High Priority)
- `-H <host>`: Remote host (control connection).
- `-l <duration>`: Test duration in seconds (positive) or transactions/bytes (negative).
- `-t <test>`: Test type (e.g., TCP_STREAM, TCP_RR).
- `-4`, `-6`: Force IPv4 or IPv6.
- `-p <port>`: Port for the control connection (default 12865).
- `-V`: Print version.
- `-h`: Print help.

### 2.3 Test-Specific Options (High Priority)
- `-m <size>`: Message/send size for stream tests.
- `-M <size>`: Receive buffer size for stream tests.
- `-r <req,res>`: Request and response sizes for RR tests.
- `-s <send,recv>`: Local socket buffer sizes.
- `-S <send,recv>`: Remote socket buffer sizes.
- `-D`: Set `TCP_NODELAY`.

## 3. Supported Test Types
1. **TCP_STREAM**: Unidirectional TCP throughput.
2. **TCP_MAERTS**: Reverse unidirectional TCP throughput (Server to Client).
3. **TCP_RR**: Synchronous TCP request/response (Transactions/s).
4. **TCP_CRR**: TCP Connect/Request/Response (New connection per transaction).
5. **TCP_CC**: TCP Connect/Close (Measure connection rate).
6. **UDP_STREAM**: Unidirectional UDP throughput.
7. **UDP_RR**: Synchronous UDP request/response.

## 4. Protocol Requirements

### 4.1 Control Connection
- Establish a TCP connection on the control port (default 12865).
- Exchange `NetperfRequest` and `NetperfResponse` structures.
- **Byte Order**: All multi-byte integers must be transmitted in **Network Byte Order (Big Endian)**.
- **Serialization**: Avoid `unsafe` memory casting. Implement safe (de)serialization.

### 4.2 Data Connection
- The control connection negotiates a dynamic data port.
- The server (`netserver_rust`) listens on the negotiated port.
- The client (`netperf_rust`) connects to the data port to perform the actual test.

## 5. Implementation & Security Requirements

### 5.1 Robustness & Safety
- **No Unsafe**: Eliminate `unsafe` blocks for network data handling. Use safe conversion methods (e.g., `to_be_bytes`, `from_be_bytes`).
- **Input Validation**:
  - Validate all command-line arguments.
  - Strictly validate all values received over the network (ports, sizes, durations) to prevent buffer overflows or logic errors.
  - Implement upper bounds for memory allocations requested by clients (prevent DoS).

### 5.2 Resource Management
- **Connection Limiting**: The server must limit the number of concurrent test sessions to prevent resource exhaustion.
- **Thread Safety**: Ensure proper synchronization if multiple threads are used.
- **Graceful Shutdown**: Handle signals and ensure connections are closed cleanly.

### 5.3 Performance
- Minimize overhead in the data path to ensure accurate benchmarking.
- Support high-performance socket options (e.g., buffer sizes, `TCP_NODELAY`).

## 6. Documentation
- Maintain a comprehensive `README.md` with usage examples.
- Document the protocol internals for future extensions.
