use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::os::raw::c_int;
use socket2::SockRef;

pub const MAXSPECDATA: usize = 162;
pub const NETPERF_PORT: u16 = 12865;

pub const NF_INET: c_int = 4;
pub const NF_INET6: c_int = 6;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NetperfRequest {
    pub request_type: c_int,
    pub dummy: c_int,
    pub test_specific_data: [c_int; MAXSPECDATA],
}

impl Default for NetperfRequest {
    fn default() -> Self {
        NetperfRequest {
            request_type: 0,
            dummy: 0,
            test_specific_data: [0; MAXSPECDATA],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NetperfResponse {
    pub response_type: c_int,
    pub serv_errno: c_int,
    pub test_specific_data: [c_int; MAXSPECDATA],
}

impl Default for NetperfResponse {
    fn default() -> Self {
        NetperfResponse {
            response_type: 0,
            serv_errno: 0,
            test_specific_data: [0; MAXSPECDATA],
        }
    }
}

pub const DO_TCP_STREAM: c_int = 10;
pub const TCP_STREAM_RESPONSE: c_int = 11;
pub const TCP_STREAM_RESULTS: c_int = 12;

pub const DO_TCP_RR: c_int = 13;
pub const TCP_RR_RESPONSE: c_int = 14;
pub const TCP_RR_RESULTS: c_int = 15;

pub const DO_UDP_STREAM: c_int = 16;
pub const UDP_STREAM_RESPONSE: c_int = 17;
pub const UDP_STREAM_RESULTS: c_int = 18;

pub const DO_UDP_RR: c_int = 19;
pub const UDP_RR_RESPONSE: c_int = 20;
pub const UDP_RR_RESULTS: c_int = 21;

pub const DO_TCP_MAERTS: c_int = 22; // Using 22 for MAERTS as in some netperf versions it's DO_DLPI_CO_STREAM but we'll use it for TCP_MAERTS
pub const TCP_MAERTS_RESPONSE: c_int = 23;
pub const TCP_MAERTS_RESULTS: c_int = 24;

pub const DO_TCP_CRR: c_int = 34;
pub const TCP_CRR_RESPONSE: c_int = 35;
pub const TCP_CRR_RESULTS: c_int = 36;

pub const DO_TCP_CC: c_int = 300;
pub const TCP_CC_RESPONSE: c_int = 301;
pub const TCP_CC_RESULTS: c_int = 302;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TcpStreamRequest {
    pub send_buf_size: c_int,
    pub recv_buf_size: c_int,
    pub receive_size: c_int,
    pub recv_alignment: c_int,
    pub recv_offset: c_int,
    pub no_delay: c_int,
    pub measure_cpu: c_int,
    pub cpu_rate: f32,
    pub test_length: c_int,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
    pub dirty_count: c_int,
    pub clean_count: c_int,
    pub port: c_int,
    pub ipfamily: c_int,
    pub recv_aio: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TcpStreamResponse {
    pub recv_buf_size: c_int,
    pub receive_size: c_int,
    pub no_delay: c_int,
    pub measure_cpu: c_int,
    pub test_length: c_int,
    pub send_buf_size: c_int,
    pub data_port_number: c_int,
    pub cpu_rate: f32,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TcpStreamResults {
    pub bytes_received: f64,
    pub recv_calls: u32,
    pub elapsed_time: f32,
    pub cpu_util: f32,
    pub serv_dem: f32,
    pub cpu_method: c_int,
    pub num_cpus: c_int,
    pub recv_buf_size: c_int,
    pub send_buf_size: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TcpRrRequest {
    pub recv_buf_size: c_int,
    pub send_buf_size: c_int,
    pub recv_alignment: c_int,
    pub recv_offset: c_int,
    pub send_alignment: c_int,
    pub send_offset: c_int,
    pub request_size: c_int,
    pub response_size: c_int,
    pub no_delay: c_int,
    pub measure_cpu: c_int,
    pub cpu_rate: f32,
    pub test_length: c_int,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
    pub port: c_int,
    pub ipfamily: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TcpRrResponse {
    pub recv_buf_size: c_int,
    pub no_delay: c_int,
    pub measure_cpu: c_int,
    pub test_length: c_int,
    pub send_buf_size: c_int,
    pub data_port_number: c_int,
    pub cpu_rate: f32,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TcpRrResults {
    pub bytes_received: u32,
    pub recv_calls: u32,
    pub trans_received: u32,
    pub elapsed_time: f32,
    pub cpu_util: f32,
    pub serv_dem: f32,
    pub cpu_method: c_int,
    pub num_cpus: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct UdpStreamRequest {
    pub recv_buf_size: c_int,
    pub message_size: c_int,
    pub recv_connected: c_int,
    pub recv_alignment: c_int,
    pub recv_offset: c_int,
    pub checksum_off: c_int,
    pub measure_cpu: c_int,
    pub cpu_rate: f32,
    pub test_length: c_int,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
    pub port: c_int,
    pub ipfamily: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct UdpStreamResponse {
    pub recv_buf_size: c_int,
    pub send_buf_size: c_int,
    pub measure_cpu: c_int,
    pub test_length: c_int,
    pub data_port_number: c_int,
    pub cpu_rate: f32,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct UdpStreamResults {
    pub messages_recvd: u32,
    pub bytes_received: u32,
    pub elapsed_time: f32,
    pub cpu_util: f32,
    pub cpu_method: c_int,
    pub num_cpus: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct UdpRrRequest {
    pub recv_buf_size: c_int,
    pub send_buf_size: c_int,
    pub recv_alignment: c_int,
    pub recv_offset: c_int,
    pub send_alignment: c_int,
    pub send_offset: c_int,
    pub request_size: c_int,
    pub response_size: c_int,
    pub no_delay: c_int,
    pub measure_cpu: c_int,
    pub cpu_rate: f32,
    pub test_length: c_int,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
    pub port: c_int,
    pub ipfamily: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct UdpRrResponse {
    pub recv_buf_size: c_int,
    pub no_delay: c_int,
    pub measure_cpu: c_int,
    pub test_length: c_int,
    pub send_buf_size: c_int,
    pub data_port_number: c_int,
    pub cpu_rate: f32,
    pub so_rcvavoid: c_int,
    pub so_sndavoid: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct UdpRrResults {
    pub bytes_received: u32,
    pub recv_calls: u32,
    pub trans_received: u32,
    pub elapsed_time: f32,
    pub cpu_util: f32,
    pub serv_dem: f32,
    pub cpu_method: c_int,
    pub num_cpus: c_int,
}

pub type TcpCcRequest = TcpRrRequest;
pub type TcpCcResponse = TcpRrResponse;
pub type TcpCcResults = TcpRrResults;

pub fn apply_tcp_options(stream: &TcpStream, send_buf: i32, recv_buf: i32, no_delay: i32) -> std::io::Result<()> {
    let sock = SockRef::from(stream);
    if send_buf > 0 {
        let _ = sock.set_send_buffer_size(send_buf as usize);
    }
    if recv_buf > 0 {
        let _ = sock.set_recv_buffer_size(recv_buf as usize);
    }
    if no_delay >= 0 {
        let _ = stream.set_nodelay(no_delay != 0);
    }
    Ok(())
}

pub fn apply_udp_options(socket: &UdpSocket, send_buf: i32, recv_buf: i32) -> std::io::Result<()> {
    let sock = SockRef::from(socket);
    if send_buf > 0 {
        let _ = sock.set_send_buffer_size(send_buf as usize);
    }
    if recv_buf > 0 {
        let _ = sock.set_recv_buffer_size(recv_buf as usize);
    }
    Ok(())
}

pub fn send_request(stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    let mut buf = [0u8; std::mem::size_of::<NetperfRequest>()];
    let ptr = req as *const NetperfRequest as *const i32;
    for i in 0..(buf.len() / 4) {
        let val = unsafe { *ptr.add(i) };
        let bytes = val.to_be_bytes();
        buf[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
    stream.write_all(&buf)
}

pub fn recv_request(stream: &mut TcpStream) -> std::io::Result<NetperfRequest> {
    let mut buf = [0u8; std::mem::size_of::<NetperfRequest>()];
    stream.read_exact(&mut buf)?;
    let mut req = NetperfRequest::default();
    let ptr = &mut req as *mut NetperfRequest as *mut i32;
    for i in 0..(buf.len() / 4) {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&buf[i * 4..(i + 1) * 4]);
        unsafe {
            *ptr.add(i) = i32::from_be_bytes(bytes);
        }
    }
    Ok(req)
}

pub fn send_response(stream: &mut TcpStream, res: &NetperfResponse) -> std::io::Result<()> {
    let mut buf = [0u8; std::mem::size_of::<NetperfResponse>()];
    let ptr = res as *const NetperfResponse as *const i32;
    for i in 0..(buf.len() / 4) {
        let val = unsafe { *ptr.add(i) };
        let bytes = val.to_be_bytes();
        buf[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
    stream.write_all(&buf)
}

pub fn recv_response(stream: &mut TcpStream) -> std::io::Result<NetperfResponse> {
    let mut buf = [0u8; std::mem::size_of::<NetperfResponse>()];
    stream.read_exact(&mut buf)?;
    let mut res = NetperfResponse::default();
    let ptr = &mut res as *mut NetperfResponse as *mut i32;
    for i in 0..(buf.len() / 4) {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&buf[i * 4..(i + 1) * 4]);
        unsafe {
            *ptr.add(i) = i32::from_be_bytes(bytes);
        }
    }
    Ok(res)
}

pub fn print_banner(name: &str) {
    println!("****************************************************************");
    println!("*                                                              *");
    let version = "0.1.0";
    let line = format!("*  {} version {}", name, version);
    let padding = 63 - line.len();
    let right_padding = " ".repeat(padding);
    println!("{}{}*", line, right_padding);
    println!("*                                                              *");
    println!("****************************************************************");
}
