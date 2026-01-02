use netperf_rust::*;
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::time::{Duration, Instant};
use std::env;
use std::thread;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let opts = parse_args(args);

    if let Some(opts) = opts {
        print_banner("netperf_rust");
        run_test(opts)
    } else {
        Ok(())
    }
}

fn print_usage() {
    println!("Usage: netperf_rust [global options] -- [test-specific options]");
    println!("\nGlobal options:");
    println!("  -H <host>       Remote host to connect to (default: localhost)");
    println!("  -l <duration>   Test duration in seconds (default: 10)");
    println!("  -t <type>       Test type: TCP_STREAM, TCP_RR, UDP_STREAM, UDP_RR, TCP_MAERTS, TCP_CRR, TCP_CC (default: TCP_STREAM)");
    println!("  -4              Use IPv4");
    println!("  -6              Use IPv6");
    println!("  -p <port>       Port for the control connection (default: 12865)");
    println!("  -V              Print version");
    println!("  -h, --help      Print this help message");
    println!("\nTest-specific options:");
    println!("  -m <size>       Message size for stream tests (default: 32768)");
    println!("  -M <size>       Remote receive buffer size for stream tests");
    println!("  -r <req,res>    Request and response sizes for RR tests (default: 1,1)");
    println!("  -s <send,recv>  Local send and receive socket buffer sizes");
    println!("  -S <send,recv>  Remote send and receive socket buffer sizes");
    println!("  -D              Set TCP_NODELAY (default: false)");
}

fn parse_args(args: Vec<String>) -> Option<TestOptions> {
    let mut host = "localhost".to_string();
    let mut duration = 10;
    let mut test_type = "TCP_STREAM".to_string();
    let mut port = NETPERF_PORT;
    let mut msg_size = 32768;
    let mut request_size = 1;
    let mut response_size = 1;
    let mut ipfamily = NF_INET;
    let mut local_send_buf = 0;
    let mut local_recv_buf = 0;
    let mut remote_send_buf = 0;
    let mut remote_recv_buf = 0;
    let mut no_delay = -1;
    let mut remote_msg_size = 0;

    let mut global_args = Vec::new();
    let mut test_args = Vec::new();
    let mut splitting = false;

    for arg in args.into_iter().skip(1) {
        if arg == "--" {
            splitting = true;
            continue;
        }
        if splitting {
            test_args.push(arg);
        } else {
            global_args.push(arg);
        }
    }

    // Parse global args
    let mut i = 0;
    while i < global_args.len() {
        match global_args[i].as_str() {
            "-H" => {
                if i + 1 < global_args.len() {
                    host = global_args[i + 1].split(',').next().unwrap_or("localhost").to_string();
                    if global_args[i + 1].contains(',') {
                        let f_part = global_args[i + 1].split(',').nth(1).unwrap_or("");
                        if f_part == "4" || f_part == "inet" { ipfamily = NF_INET; }
                        else if f_part == "6" || f_part == "inet6" { ipfamily = NF_INET6; }
                    }
                    i += 2;
                } else { i += 1; }
            }
            "-l" => {
                if i + 1 < global_args.len() {
                    duration = global_args[i+1].parse().unwrap_or(10);
                    i += 2;
                } else { i += 1; }
            }
            "-t" => {
                if i + 1 < global_args.len() {
                    test_type = global_args[i + 1].clone();
                    i += 2;
                } else { i += 1; }
            }
            "-4" => {
                ipfamily = NF_INET;
                i += 1;
            }
            "-6" => {
                ipfamily = NF_INET6;
                i += 1;
            }
            "-p" => {
                if i + 1 < global_args.len() {
                    port = global_args[i+1].parse().unwrap_or(NETPERF_PORT);
                    i += 2;
                } else { i += 1; }
            }
            "-V" => {
                println!("rust_netperf version 0.1.0 (parity with netperf 2.6.0)");
                return None;
            }
            "-h" | "--help" => {
                print_usage();
                return None;
            }
            _ => {
                println!("Unknown global argument: {}", global_args[i]);
                print_usage();
                return None;
            }
        }
    }

    // Parse test-specific args
    let mut i = 0;
    while i < test_args.len() {
        match test_args[i].as_str() {
            "-m" => {
                if i + 1 < test_args.len() {
                    msg_size = parse_units(&test_args[i+1]).unwrap_or(32768) as usize;
                    i += 2;
                } else { i += 1; }
            }
            "-M" => {
                if i + 1 < test_args.len() {
                    remote_msg_size = parse_units(&test_args[i+1]).unwrap_or(0) as i32;
                    i += 2;
                } else { i += 1; }
            }
            "-r" => {
                if i + 1 < test_args.len() {
                    let (req, res) = parse_sizespec(&test_args[i+1]);
                    if let Some(val) = req { request_size = val as i32; }
                    if let Some(val) = res { response_size = val as i32; }
                    i += 2;
                } else { i += 1; }
            }
            "-s" => {
                if i + 1 < test_args.len() {
                    let (s, r) = parse_sizespec(&test_args[i+1]);
                    if let Some(val) = s { local_send_buf = val as i32; }
                    if let Some(val) = r { local_recv_buf = val as i32; }
                    i += 2;
                } else { i += 1; }
            }
            "-S" => {
                if i + 1 < test_args.len() {
                    let (s, r) = parse_sizespec(&test_args[i+1]);
                    if let Some(val) = s { remote_send_buf = val as i32; }
                    if let Some(val) = r { remote_recv_buf = val as i32; }
                    i += 2;
                } else { i += 1; }
            }
            "-D" => {
                no_delay = 1;
                i += 1;
            }
            "-h" => {
                 // Test specific help would go here, but for now we just show general help
                 print_usage();
                 return None;
            }
            _ => {
                println!("Unknown test-specific argument: {}", test_args[i]);
                print_usage();
                return None;
            }
        }
    }

    Some(TestOptions {
        host,
        duration,
        test_type,
        port,
        msg_size,
        request_size,
        response_size,
        ipfamily,
        local_send_buf,
        local_recv_buf,
        remote_send_buf,
        remote_recv_buf,
        no_delay,
        remote_msg_size,
    })
}

struct TestOptions {
    host: String,
    duration: i32,
    test_type: String,
    port: u16,
    msg_size: usize,
    request_size: i32,
    response_size: i32,
    ipfamily: i32,
    local_send_buf: i32,
    local_recv_buf: i32,
    remote_send_buf: i32,
    remote_recv_buf: i32,
    no_delay: i32,
    remote_msg_size: i32,
}

fn run_test(opts: TestOptions) -> std::io::Result<()> {
    match opts.test_type.as_str() {
        "TCP_STREAM" => run_tcp_stream(&opts),
        "TCP_RR" => run_tcp_rr(&opts),
        "UDP_STREAM" => run_udp_stream(&opts),
        "UDP_RR" => run_udp_rr(&opts),
        "TCP_MAERTS" => run_tcp_maerts(&opts),
        "TCP_CRR" => run_tcp_crr(&opts),
        "TCP_CC" => run_tcp_cc(&opts),
        _ => {
            println!("Supported tests: TCP_STREAM, TCP_RR, UDP_STREAM, UDP_RR, TCP_MAERTS, TCP_CRR, TCP_CC");
            Ok(())
        }
    }
}

fn run_tcp_stream(opts: &TestOptions) -> std::io::Result<()> {
    let mut control_stream = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
    println!("Connected to control port on {}", opts.host);

    let mut req = NetperfRequest::default();
    req.request_type = DO_TCP_STREAM;
    
    let mut tcp_req = TcpStreamRequest::default();
    tcp_req.test_length = opts.duration;
    tcp_req.send_buf_size = opts.remote_send_buf;
    tcp_req.recv_buf_size = opts.remote_recv_buf;
    tcp_req.receive_size = if opts.remote_msg_size > 0 { opts.remote_msg_size } else { opts.msg_size as i32 };
    tcp_req.no_delay = opts.no_delay;
    tcp_req.ipfamily = opts.ipfamily;
    
    req.test_specific_data = tcp_req.to_spec_data();

    send_request(&mut control_stream, &req)?;
    let res = recv_response(&mut control_stream)?;

    if res.response_type != TCP_STREAM_RESPONSE {
        println!("Unexpected response type: {}", res.response_type);
        return Ok(());
    }

    let tcp_res = TcpStreamResponse::from_spec_data(&res.test_specific_data);

    let data_port = tcp_res.data_port_number;
    println!("Connecting to data port {}", data_port);
    let mut data_stream = TcpStream::connect(format!("{}:{}", opts.host, data_port))?;
    apply_tcp_options(&data_stream, opts.local_send_buf, opts.local_recv_buf, opts.no_delay)?;

    let start = Instant::now();
    let mut bytes_sent = 0u64;
    let buf = vec![0u8; opts.msg_size];

    let test_duration = Duration::from_secs(opts.duration as u64);
    while start.elapsed() < test_duration {
        match data_stream.write(&buf) {
            Ok(n) => {
                bytes_sent += n as u64;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(e) => return Err(e),
        }
    }
    drop(data_stream); // Close data connection

    println!("Test finished (sent {} bytes), requesting results...", bytes_sent);
    
    // Request results
    let result_req = NetperfRequest::default();
    send_request(&mut control_stream, &result_req)?;
    
    let res_final = recv_response(&mut control_stream)?;
    if res_final.response_type != TCP_STREAM_RESULTS {
        println!("Unexpected results type: {}", res_final.response_type);
        return Ok(());
    }

    let results = TcpStreamResults::from_spec_data(&res_final.test_specific_data);

    let throughput = (results.bytes_received * 8.0) / (results.elapsed_time as f64) / 1_000_000.0;
    println!("Throughput: {:.2} Mbps", throughput);
    println!("Bytes received: {}", results.bytes_received);
    println!("Elapsed time: {:.2} s", results.elapsed_time);

    Ok(())
}

fn run_tcp_rr(opts: &TestOptions) -> std::io::Result<()> {
    let mut control_stream = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
    println!("Connected to control port on {}", opts.host);

    let mut req = NetperfRequest::default();
    req.request_type = DO_TCP_RR;
    
    let mut tcp_req = TcpRrRequest::default();
    tcp_req.test_length = opts.duration;
    tcp_req.request_size = opts.request_size;
    tcp_req.response_size = opts.response_size;
    tcp_req.send_buf_size = opts.remote_send_buf;
    tcp_req.recv_buf_size = opts.remote_recv_buf;
    tcp_req.no_delay = opts.no_delay;
    tcp_req.ipfamily = opts.ipfamily;
    
    req.test_specific_data = tcp_req.to_spec_data();

    send_request(&mut control_stream, &req)?;
    let res = recv_response(&mut control_stream)?;

    if res.response_type != TCP_RR_RESPONSE {
        println!("Unexpected response type: {}", res.response_type);
        return Ok(());
    }

    let tcp_res = TcpRrResponse::from_spec_data(&res.test_specific_data);

    let data_port = tcp_res.data_port_number;
    println!("Connecting to data port {}", data_port);
    let mut data_stream = TcpStream::connect(format!("{}:{}", opts.host, data_port))?;
    apply_tcp_options(&data_stream, opts.local_send_buf, opts.local_recv_buf, opts.no_delay)?;

    let start = Instant::now();
    let mut trans_completed = 0u32;
    let mut buf = vec![0u8; std::cmp::max(opts.request_size, opts.response_size) as usize];

    let test_duration = Duration::from_secs(opts.duration as u64);
    while start.elapsed() < test_duration {
        // Send request
        if let Err(_) = data_stream.write_all(&buf[..opts.request_size as usize]) {
            break;
        }

        // Receive response
        let res_size = opts.response_size as usize;
        let mut read = 0;
        while read < res_size {
            match data_stream.read(&mut buf[read..res_size]) {
                Ok(0) => break,
                Ok(n) => read += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => break,
            }
        }
        if read < res_size { break; }
        
        trans_completed += 1;
    }
    drop(data_stream);

    println!("Test finished ({} transactions), requesting results...", trans_completed);
    
    let result_req = NetperfRequest::default();
    send_request(&mut control_stream, &result_req)?;
    
    let res_final = recv_response(&mut control_stream)?;
    if res_final.response_type != TCP_RR_RESULTS {
        println!("Unexpected results type: {}", res_final.response_type);
        return Ok(());
    }

    let results = TcpRrResults::from_spec_data(&res_final.test_specific_data);

    let trans_rate = (results.trans_received as f32) / results.elapsed_time;
    println!("Transaction rate: {:.2} Trans/s", trans_rate);
    println!("Transactions: {}", results.trans_received);
    println!("Elapsed time: {:.2} s", results.elapsed_time);

    Ok(())
}

fn run_udp_stream(opts: &TestOptions) -> std::io::Result<()> {
    let mut control_stream = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
    println!("Connected to control port on {}", opts.host);

    let mut req = NetperfRequest::default();
    req.request_type = DO_UDP_STREAM;
    
    let mut udp_req = UdpStreamRequest::default();
    udp_req.test_length = opts.duration;
    udp_req.message_size = opts.msg_size as i32;
    udp_req.recv_buf_size = opts.remote_recv_buf;
    udp_req.ipfamily = opts.ipfamily;
    
    req.test_specific_data = udp_req.to_spec_data();

    send_request(&mut control_stream, &req)?;
    let res = recv_response(&mut control_stream)?;

    if res.response_type != UDP_STREAM_RESPONSE {
        println!("Unexpected response type: {}", res.response_type);
        return Ok(());
    }

    let udp_res = UdpStreamResponse::from_spec_data(&res.test_specific_data);

    let data_port = udp_res.data_port_number;
    println!("Connecting to UDP data port {}", data_port);
    let bind_addr = if opts.ipfamily == NF_INET6 { "[::]:0" } else { "0.0.0.0:0" };
    let udp_socket = UdpSocket::bind(bind_addr)?;
    udp_socket.connect(format!("{}:{}", opts.host, data_port))?;
    apply_udp_options(&udp_socket, opts.local_send_buf, opts.local_recv_buf)?;

    let start = Instant::now();
    let mut bytes_sent = 0u64;
    let buf = vec![0u8; opts.msg_size];

    let test_duration = Duration::from_secs(opts.duration as u64);
    while start.elapsed() < test_duration {
        match udp_socket.send(&buf) {
            Ok(n) => {
                bytes_sent += n as u64;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_nanos(100));
            }
            Err(_) => break,
        }
    }

    println!("Test finished (sent {} bytes), requesting results...", bytes_sent);
    
    // Request results
    let result_req = NetperfRequest::default();
    send_request(&mut control_stream, &result_req)?;
    
    let res_final = recv_response(&mut control_stream)?;
    if res_final.response_type != UDP_STREAM_RESULTS {
        println!("Unexpected results type: {}", res_final.response_type);
        return Ok(());
    }

    let results = UdpStreamResults::from_spec_data(&res_final.test_specific_data);

    let throughput = (results.bytes_received as f64 * 8.0) / (results.elapsed_time as f64) / 1_000_000.0;
    println!("Throughput: {:.2} Mbps", throughput);
    println!("Messages received: {}", results.messages_recvd);
    println!("Bytes received: {}", results.bytes_received);
    println!("Elapsed time: {:.2} s", results.elapsed_time);

    Ok(())
}

fn run_udp_rr(opts: &TestOptions) -> std::io::Result<()> {
    let mut control_stream = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
    println!("Connected to control port on {}", opts.host);

    let mut req = NetperfRequest::default();
    req.request_type = DO_UDP_RR;
    
    let mut udp_req = UdpRrRequest::default();
    udp_req.test_length = opts.duration;
    udp_req.request_size = opts.request_size;
    udp_req.response_size = opts.response_size;
    udp_req.send_buf_size = opts.remote_send_buf;
    udp_req.recv_buf_size = opts.remote_recv_buf;
    udp_req.ipfamily = opts.ipfamily;
    
    req.test_specific_data = udp_req.to_spec_data();

    send_request(&mut control_stream, &req)?;
    let res = recv_response(&mut control_stream)?;

    if res.response_type != UDP_RR_RESPONSE {
        println!("Unexpected response type: {}", res.response_type);
        return Ok(());
    }

    let udp_res = UdpRrResponse::from_spec_data(&res.test_specific_data);

    let data_port = udp_res.data_port_number;
    println!("Connecting to UDP data port {}", data_port);
    let bind_addr = if opts.ipfamily == NF_INET6 { "[::]:0" } else { "0.0.0.0:0" };
    let udp_socket = UdpSocket::bind(bind_addr)?;
    udp_socket.connect(format!("{}:{}", opts.host, data_port))?;
    apply_udp_options(&udp_socket, opts.local_send_buf, opts.local_recv_buf)?;
    udp_socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let start = Instant::now();
    let mut trans_completed = 0u32;
    let mut buf = vec![0u8; std::cmp::max(opts.request_size, opts.response_size) as usize];

    let test_duration = Duration::from_secs(opts.duration as u64);
    while start.elapsed() < test_duration {
        // Send request
        if let Err(_) = udp_socket.send(&buf[..opts.request_size as usize]) {
            break;
        }

        // Receive response
        match udp_socket.recv(&mut buf) {
            Ok(n) if n >= opts.response_size as usize => {
                trans_completed += 1;
            }
            Ok(_) => continue,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(_) => break,
        }
    }

    println!("Test finished ({} transactions), requesting results...", trans_completed);
    
    let result_req = NetperfRequest::default();
    send_request(&mut control_stream, &result_req)?;
    
    let res_final = recv_response(&mut control_stream)?;
    if res_final.response_type != UDP_RR_RESULTS {
        println!("Unexpected results type: {}", res_final.response_type);
        return Ok(());
    }

    let results = UdpRrResults::from_spec_data(&res_final.test_specific_data);

    let trans_rate = (results.trans_received as f32) / results.elapsed_time;
    println!("Transaction rate: {:.2} Trans/s", trans_rate);
    println!("Transactions: {}", results.trans_received);
    println!("Elapsed time: {:.2} s", results.elapsed_time);

    Ok(())
}

fn run_tcp_maerts(opts: &TestOptions) -> std::io::Result<()> {
    let mut control_stream = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
    println!("Connected to control port on {}", opts.host);

    let mut req = NetperfRequest::default();
    req.request_type = DO_TCP_MAERTS;
    
    let mut tcp_req = TcpStreamRequest::default();
    tcp_req.test_length = opts.duration;
    tcp_req.send_buf_size = opts.remote_send_buf;
    tcp_req.recv_buf_size = opts.remote_recv_buf;
    tcp_req.receive_size = opts.msg_size as i32;
    tcp_req.no_delay = opts.no_delay;
    tcp_req.ipfamily = opts.ipfamily;
    
    req.test_specific_data = tcp_req.to_spec_data();

    send_request(&mut control_stream, &req)?;
    let res = recv_response(&mut control_stream)?;

    if res.response_type != TCP_MAERTS_RESPONSE {
        println!("Unexpected response type: {}", res.response_type);
        return Ok(());
    }

    let tcp_res = TcpStreamResponse::from_spec_data(&res.test_specific_data);

    let data_port = tcp_res.data_port_number;
    println!("Connecting to data port {}", data_port);
    let mut data_stream = TcpStream::connect(format!("{}:{}", opts.host, data_port))?;
    apply_tcp_options(&data_stream, opts.local_send_buf, opts.local_recv_buf, opts.no_delay)?;

    let start = Instant::now();
    let mut bytes_received = 0u64;
    let mut buf = vec![0u8; opts.msg_size];

    let test_duration = Duration::from_secs(opts.duration as u64);
    data_stream.set_read_timeout(Some(Duration::from_millis(500)))?;

    while start.elapsed() < test_duration {
        match data_stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                bytes_received += n as u64;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                continue;
            }
            Err(_) => break,
        }
    }
    drop(data_stream);

    println!("Test finished (received {} bytes), requesting results...", bytes_received);
    
    // Request results
    let result_req = NetperfRequest::default();
    send_request(&mut control_stream, &result_req)?;
    
    let res_final = recv_response(&mut control_stream)?;
    if res_final.response_type != TCP_MAERTS_RESULTS {
        println!("Unexpected results type: {}", res_final.response_type);
        return Ok(());
    }

    let results = TcpStreamResults::from_spec_data(&res_final.test_specific_data);

    let throughput = (results.bytes_received * 8.0) / (results.elapsed_time as f64) / 1_000_000.0;
    println!("Throughput: {:.2} Mbps", throughput);
    println!("Bytes sent by server: {}", results.bytes_received);
    println!("Elapsed time: {:.2} s", results.elapsed_time);

    Ok(())
}

fn run_tcp_crr(opts: &TestOptions) -> std::io::Result<()> {
    let mut control_stream = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
    println!("Connected to control port on {}", opts.host);

    let mut req = NetperfRequest::default();
    req.request_type = DO_TCP_CRR;
    
    let mut tcp_req = TcpRrRequest::default();
    tcp_req.test_length = opts.duration;
    tcp_req.request_size = opts.request_size;
    tcp_req.response_size = opts.response_size;
    tcp_req.send_buf_size = opts.remote_send_buf;
    tcp_req.recv_buf_size = opts.remote_recv_buf;
    tcp_req.no_delay = opts.no_delay;
    tcp_req.ipfamily = opts.ipfamily;
    
    req.test_specific_data = tcp_req.to_spec_data();

    send_request(&mut control_stream, &req)?;
    let res = recv_response(&mut control_stream)?;

    if res.response_type != TCP_CRR_RESPONSE {
        println!("Unexpected response type: {}", res.response_type);
        return Ok(());
    }

    let tcp_res = TcpRrResponse::from_spec_data(&res.test_specific_data);

    let data_port = tcp_res.data_port_number;
    println!("Target data port {}", data_port);

    let start = Instant::now();
    let mut trans_completed = 0u32;
    let mut buf = vec![0u8; std::cmp::max(opts.request_size, opts.response_size) as usize];

    let test_duration = Duration::from_secs(opts.duration as u64);
    while start.elapsed() < test_duration {
        // TCP_CRR: Connect, Request, Response, Close
        let mut data_stream = match TcpStream::connect(format!("{}:{}", opts.host, data_port)) {
            Ok(s) => s,
            Err(_) => {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
        };
        apply_tcp_options(&data_stream, opts.local_send_buf, opts.local_recv_buf, opts.no_delay)?;

        // Send request
        if let Err(_) = data_stream.write_all(&buf[..opts.request_size as usize]) {
            break;
        }

        // Receive response
        let res_size = opts.response_size as usize;
        let mut read = 0;
        while read < res_size {
            match data_stream.read(&mut buf[read..res_size]) {
                Ok(0) => break,
                Ok(n) => read += n,
                Err(_) => break,
            }
        }
        if read == res_size {
            trans_completed += 1;
        } else {
            break;
        }
        // data_stream dropped here (closed)
    }

    println!("Test finished ({} transactions), requesting results...", trans_completed);
    
    let result_req = NetperfRequest::default();
    send_request(&mut control_stream, &result_req)?;
    
    let res_final = recv_response(&mut control_stream)?;
    if res_final.response_type != TCP_CRR_RESULTS {
        println!("Unexpected results type: {}", res_final.response_type);
        return Ok(());
    }

    let results = TcpRrResults::from_spec_data(&res_final.test_specific_data);

    let trans_rate = (results.trans_received as f32) / results.elapsed_time;
    println!("Transaction rate: {:.2} Trans/s", trans_rate);
    println!("Transactions: {}", results.trans_received);
    println!("Elapsed time: {:.2} s", results.elapsed_time);

    Ok(())
}

fn run_tcp_cc(opts: &TestOptions) -> std::io::Result<()> {
    let mut control_stream = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
    println!("Connected to control port on {}", opts.host);

    let mut req = NetperfRequest::default();
    req.request_type = DO_TCP_CC;
    
    let mut tcp_req = TcpCcRequest::default();
    tcp_req.test_length = opts.duration;
    tcp_req.send_buf_size = opts.remote_send_buf;
    tcp_req.recv_buf_size = opts.remote_recv_buf;
    tcp_req.no_delay = opts.no_delay;
    tcp_req.ipfamily = opts.ipfamily;
    
    req.test_specific_data = tcp_req.to_spec_data();

    send_request(&mut control_stream, &req)?;
    let res = recv_response(&mut control_stream)?;

    if res.response_type != TCP_CC_RESPONSE {
        println!("Unexpected response type: {}", res.response_type);
        return Ok(());
    }

    let tcp_res = TcpCcResponse::from_spec_data(&res.test_specific_data);

    let data_port = tcp_res.data_port_number;
    println!("Target data port {}", data_port);

    let start = Instant::now();
    let mut trans_completed = 0u32;

    let test_duration = Duration::from_secs(opts.duration as u64);
    while start.elapsed() < test_duration {
        // TCP_CC: Connect, Close
        let data_stream = match TcpStream::connect(format!("{}:{}", opts.host, data_port)) {
            Ok(s) => s,
            Err(_) => {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
        };
        let _ = apply_tcp_options(&data_stream, opts.local_send_buf, opts.local_recv_buf, opts.no_delay);
        drop(data_stream);
        trans_completed += 1;
    }

    println!("Test finished ({} transactions), requesting results...", trans_completed);
    
    let result_req = NetperfRequest::default();
    send_request(&mut control_stream, &result_req)?;
    
    let res_final = recv_response(&mut control_stream)?;
    if res_final.response_type != TCP_CC_RESULTS {
        println!("Unexpected results type: {}", res_final.response_type);
        return Ok(());
    }

    let results = TcpCcResults::from_spec_data(&res_final.test_specific_data);

    let trans_rate = (results.trans_received as f32) / results.elapsed_time;
    println!("Transaction rate: {:.2} Trans/s", trans_rate);
    println!("Transactions: {}", results.trans_received);
    println!("Elapsed time: {:.2} s", results.elapsed_time);

    Ok(())
}
