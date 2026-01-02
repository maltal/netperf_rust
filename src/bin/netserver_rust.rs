use netperf_rust::*;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr};
use std::time::{Duration, Instant};
use std::thread;
use std::env;
use socket2::{Socket, Domain, Type, Protocol};

use std::sync::{Arc, Mutex};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let port = if let Some(p) = parse_args(args) {
        p
    } else {
        return Ok(());
    };

    print_banner("netserver_rust");
    let addr_v6: SocketAddr = format!("[::]:{}", port).parse().unwrap();
    let addr_v4: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    let listener = create_listener(addr_v6)
        .or_else(|_| create_listener(addr_v4))?;

    println!("Netserver listening on {}", listener.local_addr()?);

    // Security: Connection limiting
    let max_connections = 64;
    let connections = Arc::new(Mutex::new(0));

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let conn_count = connections.clone();
                {
                    let mut count = conn_count.lock().unwrap();
                    if *count >= max_connections {
                        eprintln!("Max connections reached, rejecting client");
                        continue;
                    }
                    *count += 1;
                }
                
                thread::spawn(move || {
                    if let Err(e) = handle_client(&mut stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                    let mut count = conn_count.lock().unwrap();
                    *count -= 1;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage: netserver_rust [options]");
    println!("Options:");
    println!("  -p <port>   Port to listen on (default: {})", NETPERF_PORT);
    println!("  -V          Print version");
    println!("  -h, --help  Print this help message");
}

fn parse_args(args: Vec<String>) -> Option<u16> {
    let mut port = NETPERF_PORT;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(NETPERF_PORT);
                    i += 2;
                } else { i += 1; }
            }
            "-V" => {
                println!("netserver_rust version 0.1.0");
                return None;
            }
            "-h" | "--help" => {
                print_usage();
                return None;
            }
            _ => {
                println!("Unknown argument: {}", args[i]);
                print_usage();
                return None;
            }
        }
    }
    Some(port)
}

fn create_listener(addr: SocketAddr) -> std::io::Result<TcpListener> {
    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
    if addr.is_ipv6() {
        if let Err(e) = socket.set_only_v6(false) {
            eprintln!("Warning: failed to set_only_v6(false): {}", e);
        }
    }
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;
    Ok(TcpListener::from(socket))
}

fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    loop {
        let req = match recv_request(stream) {
            Ok(req) => req,
            Err(_) => return Ok(()), // Connection closed
        };

        match req.request_type {
            DO_TCP_STREAM => {
                handle_tcp_stream(stream, &req)?;
            }
            DO_TCP_RR => {
                handle_tcp_rr(stream, &req)?;
            }
            DO_UDP_STREAM => {
                handle_udp_stream(stream, &req)?;
            }
            DO_UDP_RR => {
                handle_udp_rr(stream, &req)?;
            }
            DO_TCP_MAERTS => {
                handle_tcp_maerts(stream, &req)?;
            }
            DO_TCP_CRR => {
                handle_tcp_crr(stream, &req)?;
            }
            DO_TCP_CC => {
                handle_tcp_cc(stream, &req)?;
            }
            _ => {
                println!("Unknown request type: {}", req.request_type);
                // For now just exit or ignore
                return Ok(());
            }
        }
    }
}

fn apply_tcp_options(stream: &TcpStream, send_buf: i32, recv_buf: i32, no_delay: i32) -> std::io::Result<()> {
    netperf_rust::apply_tcp_options(stream, send_buf, recv_buf, no_delay)
}

fn apply_udp_options(socket: &UdpSocket, send_buf: i32, recv_buf: i32) -> std::io::Result<()> {
    netperf_rust::apply_udp_options(socket, send_buf, recv_buf)
}

fn handle_tcp_stream(control_stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let tcp_req = TcpStreamRequest::from_spec_data(&req.test_specific_data);

    // Set up data connection
    let local_addr = control_stream.local_addr()?;
    let bind_addr = SocketAddr::new(local_addr.ip(), 0);
    let data_listener = TcpListener::bind(bind_addr)?;
    let data_port = data_listener.local_addr()?.port();

    let mut res = NetperfResponse::default();
    res.response_type = TCP_STREAM_RESPONSE;
    let mut tcp_res = TcpStreamResponse::default();
    tcp_res.data_port_number = data_port as i32;
    tcp_res.test_length = tcp_req.test_length;
    tcp_res.recv_buf_size = tcp_req.recv_buf_size;
    tcp_res.send_buf_size = tcp_req.send_buf_size;
    tcp_res.no_delay = tcp_req.no_delay;
    
    res.test_specific_data = tcp_res.to_spec_data();

    send_response(control_stream, &res)?;

    // Accept data connection
    let (mut data_stream, _) = data_listener.accept()?;
    apply_tcp_options(&data_stream, tcp_req.send_buf_size, tcp_req.recv_buf_size, tcp_req.no_delay)?;
    
    let start = Instant::now();
    let mut bytes_received = 0u64;
    let mut recv_calls = 0u32;
    
    // Security: implement upper bounds for memory allocations
    let buf_size = if tcp_req.receive_size > 0 && tcp_req.receive_size <= 1024 * 1024 { 
        tcp_req.receive_size as usize 
    } else { 
        32768 
    };
    let mut buf = vec![0u8; buf_size];

    let test_duration = if tcp_req.test_length > 0 && tcp_req.test_length < 3600 {
        Duration::from_secs(tcp_req.test_length as u64)
    } else {
        Duration::from_secs(10) // Default
    };

    data_stream.set_read_timeout(Some(Duration::from_millis(500)))?;

    while start.elapsed() < test_duration {
        match data_stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                bytes_received += n as u64;
                recv_calls += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32();
    println!("Server: received {} bytes in {:.2}s", bytes_received, elapsed);

    // Send results
    let _result_req = match recv_request(control_stream) {
        Ok(req) => req,
        Err(e) => return Err(e),
    };

    let mut results = TcpStreamResults::default();
    results.bytes_received = bytes_received as f64;
    results.recv_calls = recv_calls;
    results.elapsed_time = elapsed;
    
    let mut res_final = NetperfResponse::default();
    res_final.response_type = TCP_STREAM_RESULTS;
    res_final.test_specific_data = results.to_spec_data();
    
    send_response(control_stream, &res_final)?;

    Ok(())
}

fn handle_tcp_rr(control_stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let tcp_req = TcpRrRequest::from_spec_data(&req.test_specific_data);

    let local_addr = control_stream.local_addr()?;
    let bind_addr = SocketAddr::new(local_addr.ip(), 0);
    let data_listener = TcpListener::bind(bind_addr)?;
    let data_port = data_listener.local_addr()?.port();

    let mut res = NetperfResponse::default();
    res.response_type = TCP_RR_RESPONSE;
    let mut tcp_res = TcpRrResponse::default();
    tcp_res.data_port_number = data_port as i32;
    tcp_res.test_length = tcp_req.test_length;
    tcp_res.recv_buf_size = tcp_req.recv_buf_size;
    tcp_res.send_buf_size = tcp_req.send_buf_size;
    tcp_res.no_delay = tcp_req.no_delay;
    
    res.test_specific_data = tcp_res.to_spec_data();

    send_response(control_stream, &res)?;

    let (mut data_stream, _) = data_listener.accept()?;
    apply_tcp_options(&data_stream, tcp_req.send_buf_size, tcp_req.recv_buf_size, tcp_req.no_delay)?;
    
    let start = Instant::now();
    let mut trans_received = 0u32;
    
    // Security: Input validation
    let req_size = if tcp_req.request_size > 0 && tcp_req.request_size <= 1024 * 1024 {
        tcp_req.request_size as usize
    } else {
        1
    };
    let res_size = if tcp_req.response_size > 0 && tcp_req.response_size <= 1024 * 1024 {
        tcp_req.response_size as usize
    } else {
        1
    };
    
    let mut buf = vec![0u8; std::cmp::max(req_size, res_size)];

    let test_duration = if tcp_req.test_length > 0 && tcp_req.test_length < 3600 {
        Duration::from_secs(tcp_req.test_length as u64)
    } else {
        Duration::from_secs(10)
    };

    while start.elapsed() < test_duration {
        // Read request
        let mut read = 0;
        while read < req_size {
            match data_stream.read(&mut buf[read..req_size]) {
                Ok(0) => break,
                Ok(n) => read += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => break,
            }
        }
        if read < req_size { break; }

        // Send response
        if let Err(_) = data_stream.write_all(&buf[..res_size]) {
            break;
        }
        trans_received += 1;
    }
    
    let elapsed = start.elapsed().as_secs_f32();

    let _result_req = match recv_request(control_stream) {
        Ok(req) => req,
        Err(e) => return Err(e),
    };

    let mut results = TcpRrResults::default();
    results.trans_received = trans_received;
    results.elapsed_time = elapsed;
    
    let mut res_final = NetperfResponse::default();
    res_final.response_type = TCP_RR_RESULTS;
    res_final.test_specific_data = results.to_spec_data();
    
    send_response(control_stream, &res_final)?;

    Ok(())
}

fn handle_udp_stream(control_stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let udp_req = UdpStreamRequest::from_spec_data(&req.test_specific_data);

    let local_addr = control_stream.local_addr()?;
    let bind_addr = SocketAddr::new(local_addr.ip(), 0);
    let udp_socket = UdpSocket::bind(bind_addr)?;
    apply_udp_options(&udp_socket, 0, udp_req.recv_buf_size)?;
    let data_port = udp_socket.local_addr()?.port();

    let mut res = NetperfResponse::default();
    res.response_type = UDP_STREAM_RESPONSE;
    let mut udp_res = UdpStreamResponse::default();
    udp_res.data_port_number = data_port as i32;
    udp_res.test_length = udp_req.test_length;
    udp_res.recv_buf_size = udp_req.recv_buf_size;
    
    res.test_specific_data = udp_res.to_spec_data();

    send_response(control_stream, &res)?;

    let start = Instant::now();
    let mut bytes_received = 0u64;
    let mut messages_recvd = 0u32;
    let mut buf = vec![0u8; 65536];

    let test_duration = if udp_req.test_length > 0 && udp_req.test_length < 3600 {
        Duration::from_secs(udp_req.test_length as u64)
    } else {
        Duration::from_secs(10)
    };

    udp_socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    while start.elapsed() < test_duration {
        match udp_socket.recv_from(&mut buf) {
            Ok((n, _addr)) => {
                bytes_received += n as u64;
                messages_recvd += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32();
    println!("Server: UDP received {} bytes in {:.2}s", bytes_received, elapsed);

    let _result_req = match recv_request(control_stream) {
        Ok(req) => req,
        Err(e) => return Err(e),
    };

    let mut results = UdpStreamResults::default();
    results.bytes_received = bytes_received as u32;
    results.messages_recvd = messages_recvd;
    results.elapsed_time = elapsed;
    
    let mut res_final = NetperfResponse::default();
    res_final.response_type = UDP_STREAM_RESULTS;
    res_final.test_specific_data = results.to_spec_data();
    
    send_response(control_stream, &res_final)?;

    Ok(())
}

fn handle_udp_rr(control_stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let udp_req = UdpRrRequest::from_spec_data(&req.test_specific_data);

    let local_addr = control_stream.local_addr()?;
    let bind_addr = SocketAddr::new(local_addr.ip(), 0);
    let udp_socket = UdpSocket::bind(bind_addr)?;
    apply_udp_options(&udp_socket, udp_req.send_buf_size, udp_req.recv_buf_size)?;
    let data_port = udp_socket.local_addr()?.port();

    let mut res = NetperfResponse::default();
    res.response_type = UDP_RR_RESPONSE;
    let mut udp_res = UdpRrResponse::default();
    udp_res.data_port_number = data_port as i32;
    udp_res.test_length = udp_req.test_length;
    udp_res.recv_buf_size = udp_req.recv_buf_size;
    udp_res.send_buf_size = udp_req.send_buf_size;
    udp_res.no_delay = udp_req.no_delay;
    
    res.test_specific_data = udp_res.to_spec_data();

    send_response(control_stream, &res)?;

    let start = Instant::now();
    let mut trans_received = 0u32;
    
    // Security: Input validation
    let req_size = if udp_req.request_size > 0 && udp_req.request_size <= 65535 {
        udp_req.request_size as usize
    } else {
        1
    };
    let res_size = if udp_req.response_size > 0 && udp_req.response_size <= 65535 {
        udp_req.response_size as usize
    } else {
        1
    };
    let mut buf = vec![0u8; std::cmp::max(req_size, res_size)];

    let test_duration = if udp_req.test_length > 0 && udp_req.test_length < 3600 {
        Duration::from_secs(udp_req.test_length as u64)
    } else {
        Duration::from_secs(10)
    };

    udp_socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    while start.elapsed() < test_duration {
        // Read request
        match udp_socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                if n >= req_size {
                    // Send response
                    if let Err(_) = udp_socket.send_to(&buf[..res_size], addr) {
                        break;
                    }
                    trans_received += 1;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                continue;
            }
            Err(_) => break,
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32();

    let _result_req = match recv_request(control_stream) {
        Ok(req) => req,
        Err(e) => return Err(e),
    };

    let mut results = UdpRrResults::default();
    results.trans_received = trans_received;
    results.elapsed_time = elapsed;
    
    let mut res_final = NetperfResponse::default();
    res_final.response_type = UDP_RR_RESULTS;
    res_final.test_specific_data = results.to_spec_data();
    
    send_response(control_stream, &res_final)?;

    Ok(())
}

fn handle_tcp_maerts(control_stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let tcp_req = TcpStreamRequest::from_spec_data(&req.test_specific_data);

    let local_addr = control_stream.local_addr()?;
    let bind_addr = SocketAddr::new(local_addr.ip(), 0);
    let data_listener = TcpListener::bind(bind_addr)?;
    let data_port = data_listener.local_addr()?.port();

    let mut res = NetperfResponse::default();
    res.response_type = TCP_MAERTS_RESPONSE;
    let mut tcp_res = TcpStreamResponse::default();
    tcp_res.data_port_number = data_port as i32;
    tcp_res.test_length = tcp_req.test_length;
    tcp_res.recv_buf_size = tcp_req.recv_buf_size;
    tcp_res.send_buf_size = tcp_req.send_buf_size;
    tcp_res.no_delay = tcp_req.no_delay;
    
    res.test_specific_data = tcp_res.to_spec_data();

    send_response(control_stream, &res)?;

    let (mut data_stream, _) = data_listener.accept()?;
    apply_tcp_options(&data_stream, tcp_req.send_buf_size, tcp_req.recv_buf_size, tcp_req.no_delay)?;
    
    let start = Instant::now();
    let mut bytes_sent = 0u64;
    let mut send_calls = 0u32;
    
    // Security: implement upper bounds for memory allocations
    let buf_size = if tcp_req.receive_size > 0 && tcp_req.receive_size <= 1024 * 1024 { 
        tcp_req.receive_size as usize 
    } else { 
        32768 
    };
    let buf = vec![0u8; buf_size];

    let test_duration = if tcp_req.test_length > 0 && tcp_req.test_length < 3600 {
        Duration::from_secs(tcp_req.test_length as u64)
    } else {
        Duration::from_secs(10)
    };

    while start.elapsed() < test_duration {
        match data_stream.write(&buf) {
            Ok(n) => {
                bytes_sent += n as u64;
                send_calls += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(_) => break,
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32();

    let _result_req = match recv_request(control_stream) {
        Ok(req) => req,
        Err(e) => return Err(e),
    };

    let mut results = TcpStreamResults::default();
    results.bytes_received = bytes_sent as f64;
    results.recv_calls = send_calls;
    results.elapsed_time = elapsed;
    
    let mut res_final = NetperfResponse::default();
    res_final.response_type = TCP_MAERTS_RESULTS;
    res_final.test_specific_data = results.to_spec_data();
    
    send_response(control_stream, &res_final)?;

    Ok(())
}

fn handle_tcp_crr(control_stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let tcp_req = TcpRrRequest::from_spec_data(&req.test_specific_data);

    let local_addr = control_stream.local_addr()?;
    let bind_addr = SocketAddr::new(local_addr.ip(), 0);
    let data_listener = TcpListener::bind(bind_addr)?;
    data_listener.set_nonblocking(true)?;
    let data_port = data_listener.local_addr()?.port();

    let mut res = NetperfResponse::default();
    res.response_type = TCP_CRR_RESPONSE;
    let mut tcp_res = TcpRrResponse::default();
    tcp_res.data_port_number = data_port as i32;
    tcp_res.test_length = tcp_req.test_length;
    tcp_res.recv_buf_size = tcp_req.recv_buf_size;
    tcp_res.send_buf_size = tcp_req.send_buf_size;
    tcp_res.no_delay = tcp_req.no_delay;
    
    res.test_specific_data = tcp_res.to_spec_data();

    send_response(control_stream, &res)?;

    let start = Instant::now();
    let mut trans_received = 0u32;
    
    // Security: Input validation
    let req_size = if tcp_req.request_size > 0 && tcp_req.request_size <= 1024 * 1024 {
        tcp_req.request_size as usize
    } else {
        1
    };
    let res_size = if tcp_req.response_size > 0 && tcp_req.response_size <= 1024 * 1024 {
        tcp_req.response_size as usize
    } else {
        1
    };
    let mut buf = vec![0u8; std::cmp::max(req_size, res_size)];

    let test_duration = if tcp_req.test_length > 0 && tcp_req.test_length < 3600 {
        Duration::from_secs(tcp_req.test_length as u64)
    } else {
        Duration::from_secs(10)
    };

    while start.elapsed() < test_duration {
        // TCP_CRR accepts a new connection for EACH transaction
        let (mut data_stream, _) = match data_listener.accept() {
            Ok(s) => s,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(_) => break,
        };
        let _ = apply_tcp_options(&data_stream, tcp_req.send_buf_size, tcp_req.recv_buf_size, tcp_req.no_delay);

        // Read request
        let mut read = 0;
        while read < req_size {
            match data_stream.read(&mut buf[read..req_size]) {
                Ok(0) => break,
                Ok(n) => read += n,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => break,
            }
        }
        
        if read == req_size {
            // Send response
            if let Ok(_) = data_stream.write_all(&buf[..res_size]) {
                trans_received += 1;
            }
        }
        // Connection closed after one transaction in CRR
    }
    
    let elapsed = start.elapsed().as_secs_f32();

    let _result_req = match recv_request(control_stream) {
        Ok(req) => req,
        Err(e) => return Err(e),
    };

    let mut results = TcpRrResults::default();
    results.trans_received = trans_received;
    results.elapsed_time = elapsed;
    
    let mut res_final = NetperfResponse::default();
    res_final.response_type = TCP_CRR_RESULTS;
    res_final.test_specific_data = results.to_spec_data();
    
    send_response(control_stream, &res_final)?;

    Ok(())
}

fn handle_tcp_cc(control_stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let tcp_req = TcpCcRequest::from_spec_data(&req.test_specific_data);

    let local_addr = control_stream.local_addr()?;
    let bind_addr = SocketAddr::new(local_addr.ip(), 0);
    let data_listener = TcpListener::bind(bind_addr)?;
    data_listener.set_nonblocking(true)?;
    let data_port = data_listener.local_addr()?.port();

    let mut res = NetperfResponse::default();
    res.response_type = TCP_CC_RESPONSE;
    let mut tcp_res = TcpCcResponse::default();
    tcp_res.data_port_number = data_port as i32;
    tcp_res.test_length = tcp_req.test_length;
    tcp_res.recv_buf_size = tcp_req.recv_buf_size;
    tcp_res.send_buf_size = tcp_req.send_buf_size;
    tcp_res.no_delay = tcp_req.no_delay;
    
    res.test_specific_data = tcp_res.to_spec_data();

    send_response(control_stream, &res)?;

    let start = Instant::now();
    let mut trans_received = 0u32;

    let test_duration = if tcp_req.test_length > 0 && tcp_req.test_length < 3600 {
        Duration::from_secs(tcp_req.test_length as u64)
    } else {
        Duration::from_secs(10)
    };

    while start.elapsed() < test_duration {
        // TCP_CC accepts and closes immediately
        let (data_stream, _) = match data_listener.accept() {
            Ok(s) => s,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(_) => break,
        };
        let _ = apply_tcp_options(&data_stream, tcp_req.send_buf_size, tcp_req.recv_buf_size, tcp_req.no_delay);
        drop(data_stream);
        trans_received += 1;
    }
    
    let elapsed = start.elapsed().as_secs_f32();

    // Send results
    let _result_req = recv_request(control_stream)?;

    let mut results = TcpCcResults::default();
    results.trans_received = trans_received;
    results.elapsed_time = elapsed;
    
    let mut res_final = NetperfResponse::default();
    res_final.response_type = TCP_CC_RESULTS;
    res_final.test_specific_data = results.to_spec_data();
    
    send_response(control_stream, &res_final)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_listener() {
        // Test binding to an ephemeral port on localhost
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = create_listener(addr).expect("Failed to create listener");
        assert!(listener.local_addr().unwrap().port() > 0);
    }

    #[test]
    fn test_create_listener_v6() {
        // Test binding to an ephemeral port on [::1]
        let addr: SocketAddr = "[::1]:0".parse().unwrap();
        if let Ok(listener) = create_listener(addr) {
             assert!(listener.local_addr().unwrap().port() > 0);
        } else {
            // IPv6 might not be supported in all environments, so we don't necessarily fail here
            eprintln!("Skipping IPv6 test as it's not supported");
        }
    }
}
