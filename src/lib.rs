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

impl NetperfRequest {
    pub fn to_bytes(&self) -> [u8; 8 + MAXSPECDATA * 4] {
        let mut buf = [0u8; 8 + MAXSPECDATA * 4];
        buf[0..4].copy_from_slice(&self.request_type.to_be_bytes());
        buf[4..8].copy_from_slice(&self.dummy.to_be_bytes());
        for i in 0..MAXSPECDATA {
            buf[8 + i * 4..12 + i * 4].copy_from_slice(&self.test_specific_data[i].to_be_bytes());
        }
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        let mut req = NetperfRequest::default();
        req.request_type = i32::from_be_bytes(buf[0..4].try_into().unwrap());
        req.dummy = i32::from_be_bytes(buf[4..8].try_into().unwrap());
        for i in 0..MAXSPECDATA {
            req.test_specific_data[i] = i32::from_be_bytes(buf[8 + i * 4..12 + i * 4].try_into().unwrap());
        }
        req
    }
}

impl NetperfResponse {
    pub fn to_bytes(&self) -> [u8; 8 + MAXSPECDATA * 4] {
        let mut buf = [0u8; 8 + MAXSPECDATA * 4];
        buf[0..4].copy_from_slice(&self.response_type.to_be_bytes());
        buf[4..8].copy_from_slice(&self.serv_errno.to_be_bytes());
        for i in 0..MAXSPECDATA {
            buf[8 + i * 4..12 + i * 4].copy_from_slice(&self.test_specific_data[i].to_be_bytes());
        }
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        let mut res = NetperfResponse::default();
        res.response_type = i32::from_be_bytes(buf[0..4].try_into().unwrap());
        res.serv_errno = i32::from_be_bytes(buf[4..8].try_into().unwrap());
        for i in 0..MAXSPECDATA {
            res.test_specific_data[i] = i32::from_be_bytes(buf[8 + i * 4..12 + i * 4].try_into().unwrap());
        }
        res
    }
}

impl TcpStreamRequest {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.send_buf_size;
        data[1] = self.recv_buf_size;
        data[2] = self.receive_size;
        data[3] = self.recv_alignment;
        data[4] = self.recv_offset;
        data[5] = self.no_delay;
        data[6] = self.measure_cpu;
        data[7] = self.cpu_rate.to_bits() as i32;
        data[8] = self.test_length;
        data[9] = self.so_rcvavoid;
        data[10] = self.so_sndavoid;
        data[11] = self.dirty_count;
        data[12] = self.clean_count;
        data[13] = self.port;
        data[14] = self.ipfamily;
        data[15] = self.recv_aio;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            send_buf_size: data[0],
            recv_buf_size: data[1],
            receive_size: data[2],
            recv_alignment: data[3],
            recv_offset: data[4],
            no_delay: data[5],
            measure_cpu: data[6],
            cpu_rate: f32::from_bits(data[7] as u32),
            test_length: data[8],
            so_rcvavoid: data[9],
            so_sndavoid: data[10],
            dirty_count: data[11],
            clean_count: data[12],
            port: data[13],
            ipfamily: data[14],
            recv_aio: data[15],
        }
    }
}

impl TcpStreamResponse {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.recv_buf_size;
        data[1] = self.receive_size;
        data[2] = self.no_delay;
        data[3] = self.measure_cpu;
        data[4] = self.test_length;
        data[5] = self.send_buf_size;
        data[6] = self.data_port_number;
        data[7] = self.cpu_rate.to_bits() as i32;
        data[8] = self.so_rcvavoid;
        data[9] = self.so_sndavoid;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            recv_buf_size: data[0],
            receive_size: data[1],
            no_delay: data[2],
            measure_cpu: data[3],
            test_length: data[4],
            send_buf_size: data[5],
            data_port_number: data[6],
            cpu_rate: f32::from_bits(data[7] as u32),
            so_rcvavoid: data[8],
            so_sndavoid: data[9],
        }
    }
}

impl TcpStreamResults {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        let bytes_bits = self.bytes_received.to_bits();
        data[0] = (bytes_bits & 0xFFFFFFFF) as i32;
        data[1] = (bytes_bits >> 32) as i32;
        data[2] = self.recv_calls as i32;
        data[3] = self.elapsed_time.to_bits() as i32;
        data[4] = self.cpu_util.to_bits() as i32;
        data[5] = self.serv_dem.to_bits() as i32;
        data[6] = self.cpu_method;
        data[7] = self.num_cpus;
        data[8] = self.recv_buf_size;
        data[9] = self.send_buf_size;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        let bytes_bits = (data[0] as u32 as u64) | ((data[1] as u32 as u64) << 32);
        Self {
            bytes_received: f64::from_bits(bytes_bits),
            recv_calls: data[2] as u32,
            elapsed_time: f32::from_bits(data[3] as u32),
            cpu_util: f32::from_bits(data[4] as u32),
            serv_dem: f32::from_bits(data[5] as u32),
            cpu_method: data[6],
            num_cpus: data[7],
            recv_buf_size: data[8],
            send_buf_size: data[9],
        }
    }
}

impl TcpRrRequest {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.recv_buf_size;
        data[1] = self.send_buf_size;
        data[2] = self.recv_alignment;
        data[3] = self.recv_offset;
        data[4] = self.send_alignment;
        data[5] = self.send_offset;
        data[6] = self.request_size;
        data[7] = self.response_size;
        data[8] = self.no_delay;
        data[9] = self.measure_cpu;
        data[10] = self.cpu_rate.to_bits() as i32;
        data[11] = self.test_length;
        data[12] = self.so_rcvavoid;
        data[13] = self.so_sndavoid;
        data[14] = self.port;
        data[15] = self.ipfamily;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            recv_buf_size: data[0],
            send_buf_size: data[1],
            recv_alignment: data[2],
            recv_offset: data[3],
            send_alignment: data[4],
            send_offset: data[5],
            request_size: data[6],
            response_size: data[7],
            no_delay: data[8],
            measure_cpu: data[9],
            cpu_rate: f32::from_bits(data[10] as u32),
            test_length: data[11],
            so_rcvavoid: data[12],
            so_sndavoid: data[13],
            port: data[14],
            ipfamily: data[15],
        }
    }
}

impl TcpRrResponse {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.recv_buf_size;
        data[1] = self.no_delay;
        data[2] = self.measure_cpu;
        data[3] = self.test_length;
        data[4] = self.send_buf_size;
        data[5] = self.data_port_number;
        data[6] = self.cpu_rate.to_bits() as i32;
        data[7] = self.so_rcvavoid;
        data[8] = self.so_sndavoid;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            recv_buf_size: data[0],
            no_delay: data[1],
            measure_cpu: data[2],
            test_length: data[3],
            send_buf_size: data[4],
            data_port_number: data[5],
            cpu_rate: f32::from_bits(data[6] as u32),
            so_rcvavoid: data[7],
            so_sndavoid: data[8],
        }
    }
}

impl TcpRrResults {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.bytes_received as i32;
        data[1] = self.recv_calls as i32;
        data[2] = self.trans_received as i32;
        data[3] = self.elapsed_time.to_bits() as i32;
        data[4] = self.cpu_util.to_bits() as i32;
        data[5] = self.serv_dem.to_bits() as i32;
        data[6] = self.cpu_method;
        data[7] = self.num_cpus;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            bytes_received: data[0] as u32,
            recv_calls: data[1] as u32,
            trans_received: data[2] as u32,
            elapsed_time: f32::from_bits(data[3] as u32),
            cpu_util: f32::from_bits(data[4] as u32),
            serv_dem: f32::from_bits(data[5] as u32),
            cpu_method: data[6],
            num_cpus: data[7],
        }
    }
}

impl UdpStreamRequest {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.recv_buf_size;
        data[1] = self.message_size;
        data[2] = self.recv_connected;
        data[3] = self.recv_alignment;
        data[4] = self.recv_offset;
        data[5] = self.checksum_off;
        data[6] = self.measure_cpu;
        data[7] = self.cpu_rate.to_bits() as i32;
        data[8] = self.test_length;
        data[9] = self.so_rcvavoid;
        data[10] = self.so_sndavoid;
        data[11] = self.port;
        data[12] = self.ipfamily;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            recv_buf_size: data[0],
            message_size: data[1],
            recv_connected: data[2],
            recv_alignment: data[3],
            recv_offset: data[4],
            checksum_off: data[5],
            measure_cpu: data[6],
            cpu_rate: f32::from_bits(data[7] as u32),
            test_length: data[8],
            so_rcvavoid: data[9],
            so_sndavoid: data[10],
            port: data[11],
            ipfamily: data[12],
        }
    }
}

impl UdpStreamResponse {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.recv_buf_size;
        data[1] = self.send_buf_size;
        data[2] = self.measure_cpu;
        data[3] = self.test_length;
        data[4] = self.data_port_number;
        data[5] = self.cpu_rate.to_bits() as i32;
        data[6] = self.so_rcvavoid;
        data[7] = self.so_sndavoid;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            recv_buf_size: data[0],
            send_buf_size: data[1],
            measure_cpu: data[2],
            test_length: data[3],
            data_port_number: data[4],
            cpu_rate: f32::from_bits(data[5] as u32),
            so_rcvavoid: data[6],
            so_sndavoid: data[7],
        }
    }
}

impl UdpStreamResults {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.messages_recvd as i32;
        data[1] = self.bytes_received as i32;
        data[2] = self.elapsed_time.to_bits() as i32;
        data[3] = self.cpu_util.to_bits() as i32;
        data[4] = self.cpu_method;
        data[5] = self.num_cpus;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            messages_recvd: data[0] as u32,
            bytes_received: data[1] as u32,
            elapsed_time: f32::from_bits(data[2] as u32),
            cpu_util: f32::from_bits(data[3] as u32),
            cpu_method: data[4],
            num_cpus: data[5],
        }
    }
}

impl UdpRrRequest {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.recv_buf_size;
        data[1] = self.send_buf_size;
        data[2] = self.recv_alignment;
        data[3] = self.recv_offset;
        data[4] = self.send_alignment;
        data[5] = self.send_offset;
        data[6] = self.request_size;
        data[7] = self.response_size;
        data[8] = self.no_delay;
        data[9] = self.measure_cpu;
        data[10] = self.cpu_rate.to_bits() as i32;
        data[11] = self.test_length;
        data[12] = self.so_rcvavoid;
        data[13] = self.so_sndavoid;
        data[14] = self.port;
        data[15] = self.ipfamily;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            recv_buf_size: data[0],
            send_buf_size: data[1],
            recv_alignment: data[2],
            recv_offset: data[3],
            send_alignment: data[4],
            send_offset: data[5],
            request_size: data[6],
            response_size: data[7],
            no_delay: data[8],
            measure_cpu: data[9],
            cpu_rate: f32::from_bits(data[10] as u32),
            test_length: data[11],
            so_rcvavoid: data[12],
            so_sndavoid: data[13],
            port: data[14],
            ipfamily: data[15],
        }
    }
}

impl UdpRrResponse {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.recv_buf_size;
        data[1] = self.no_delay;
        data[2] = self.measure_cpu;
        data[3] = self.test_length;
        data[4] = self.send_buf_size;
        data[5] = self.data_port_number;
        data[6] = self.cpu_rate.to_bits() as i32;
        data[7] = self.so_rcvavoid;
        data[8] = self.so_sndavoid;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            recv_buf_size: data[0],
            no_delay: data[1],
            measure_cpu: data[2],
            test_length: data[3],
            send_buf_size: data[4],
            data_port_number: data[5],
            cpu_rate: f32::from_bits(data[6] as u32),
            so_rcvavoid: data[7],
            so_sndavoid: data[8],
        }
    }
}

impl UdpRrResults {
    pub fn to_spec_data(&self) -> [c_int; MAXSPECDATA] {
        let mut data = [0; MAXSPECDATA];
        data[0] = self.bytes_received as i32;
        data[1] = self.recv_calls as i32;
        data[2] = self.trans_received as i32;
        data[3] = self.elapsed_time.to_bits() as i32;
        data[4] = self.cpu_util.to_bits() as i32;
        data[5] = self.serv_dem.to_bits() as i32;
        data[6] = self.cpu_method;
        data[7] = self.num_cpus;
        data
    }

    pub fn from_spec_data(data: &[c_int; MAXSPECDATA]) -> Self {
        Self {
            bytes_received: data[0] as u32,
            recv_calls: data[1] as u32,
            trans_received: data[2] as u32,
            elapsed_time: f32::from_bits(data[3] as u32),
            cpu_util: f32::from_bits(data[4] as u32),
            serv_dem: f32::from_bits(data[5] as u32),
            cpu_method: data[6],
            num_cpus: data[7],
        }
    }
}

pub fn send_request(stream: &mut TcpStream, req: &NetperfRequest) -> std::io::Result<()> {
    stream.write_all(&req.to_bytes())
}

pub fn recv_request(stream: &mut TcpStream) -> std::io::Result<NetperfRequest> {
    let mut buf = [0u8; 8 + MAXSPECDATA * 4];
    stream.read_exact(&mut buf)?;
    Ok(NetperfRequest::from_bytes(&buf))
}

pub fn send_response(stream: &mut TcpStream, res: &NetperfResponse) -> std::io::Result<()> {
    stream.write_all(&res.to_bytes())
}

pub fn recv_response(stream: &mut TcpStream) -> std::io::Result<NetperfResponse> {
    let mut buf = [0u8; 8 + MAXSPECDATA * 4];
    stream.read_exact(&mut buf)?;
    Ok(NetperfResponse::from_bytes(&buf))
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

pub fn parse_units(s: &str) -> Option<u64> {
    let mut num_str = s;
    let mut multiplier: u64 = 1;

    if s.is_empty() {
        return None;
    }

    let last_char = s.chars().last().unwrap();
    match last_char {
        'K' => {
            multiplier = 1024;
            num_str = &s[..s.len() - 1];
        }
        'M' => {
            multiplier = 1024 * 1024;
            num_str = &s[..s.len() - 1];
        }
        'G' => {
            multiplier = 1024 * 1024 * 1024;
            num_str = &s[..s.len() - 1];
        }
        'k' => {
            multiplier = 1000;
            num_str = &s[..s.len() - 1];
        }
        'm' => {
            multiplier = 1000 * 1000;
            num_str = &s[..s.len() - 1];
        }
        'g' => {
            multiplier = 1000 * 1000 * 1000;
            num_str = &s[..s.len() - 1];
        }
        _ => {
            if !last_char.is_digit(10) {
                // Unknown suffix
                return None;
            }
        }
    }

    num_str.parse::<u64>().ok().map(|n| {
        if last_char == 'k' || last_char == 'm' || last_char == 'g' {
             n * multiplier / 8 // bits to bytes
        } else {
             n * multiplier
        }
    })
}

pub fn parse_sizespec(s: &str) -> (Option<u64>, Option<u64>) {
    if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        let p1 = if !parts[0].is_empty() { parse_units(parts[0]) } else { None };
        let p2 = if parts.len() > 1 && !parts[1].is_empty() { parse_units(parts[1]) } else { None };
        (p1, p2)
    } else {
        let val = parse_units(s);
        (val, val)
    }
}

pub fn parse_optionspec(s: &str) -> (Option<u64>, Option<u64>) {
    if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        let p1 = if !parts[0].is_empty() { parse_units(parts[0]) } else { None };
        let p2 = if parts.len() > 1 && !parts[1].is_empty() { parse_units(parts[1]) } else { None };
        (p1, p2)
    } else {
        let val = parse_units(s);
        (val, None)
    }
}
